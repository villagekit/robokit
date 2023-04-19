// https://github.com/robert-budde/iHSV-Servo-Tool/blob/master/iHSV_Properties.py

use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use embedded_hal::serial::{Read, Write};
use heapless::{Deque, Vec};
use num::abs;

use super::Actuator;
use crate::error::Error;
use crate::modbus::{ModbusSerial, ModbusSerialError, ModbusSerialErrorAlias};
use crate::util::{i16_to_u16, u16_to_i16};

static ACCELERATION_IN_MS_PER_1000_RPM: u16 = 10_000;

#[derive(Clone, Copy, Debug, Format)]
pub enum SpindleAction {
    Set { status: SpindleStatus },
}

#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum SpindleStatus {
    Off,
    On { rpm: i16 },
}

pub trait SpindleDriver {
    type Error: Error;

    fn set(&mut self, status: SpindleStatus);
    fn poll(&mut self) -> Poll<Result<(), Self::Error>>;
}

#[derive(Clone, Copy, Debug, Format)]
enum JmcHsv57ModbusRequest {
    InitControlMode,
    InitSpeedSource,
    InitAcceleration { ms_per_1000_rpm: u16 },
    InitDeceleration { ms_per_1000_rpm: u16 },
    SetSpeed { rpm: u16 },
    GetSpeed,
}

#[derive(Clone, Copy, Debug, Format)]
enum JmcHsv57ModbusResponseType {
    InitControlMode,
    InitSpeedSource,
    InitAcceleration,
    InitDeceleration,
    SetSpeed,
    GetSpeed,
}

pub struct SpindleDriverJmcHsv57<Serial>
where
    Serial: Write<u8> + Read<u8>,
{
    modbus: ModbusSerial<Serial>,
    modbus_requests: Deque<JmcHsv57ModbusRequest, 8>,
    modbus_response_type: Option<JmcHsv57ModbusResponseType>,
    has_initialized: bool,
    spindle_status: SpindleStatus,
    next_spindle_status: Option<SpindleStatus>,
    current_rpm: Option<i16>,
}

const RPM_ERROR_BOUND: i16 = 2;

impl<Serial> SpindleDriverJmcHsv57<Serial>
where
    Serial: Write<u8> + Read<u8>,
    <Serial as Write<u8>>::Error: Debug,
    <Serial as Read<u8>>::Error: Debug,
{
    pub fn new(serial: Serial) -> Self {
        Self {
            modbus: ModbusSerial::new(serial, 1),
            modbus_requests: Deque::new(),
            modbus_response_type: None,
            has_initialized: false,
            spindle_status: SpindleStatus::Off,
            next_spindle_status: None,
            current_rpm: None,
        }
    }

    // https://www.makerstore.com.au/download/software/Manual-of-IHSV-Integrated-servo-motor.pdf
    pub fn handle_modbus(&mut self) -> Poll<Result<(), ModbusSerialErrorAlias<Serial>>> {
        match self.modbus.poll() {
            Poll::Ready(Ok(is_response_ready)) => {
                if is_response_ready {
                    // handle modbus response
                    match self.modbus_response_type.unwrap() {
                        JmcHsv57ModbusResponseType::InitControlMode => self.modbus.parse_ok()?,
                        JmcHsv57ModbusResponseType::InitSpeedSource => self.modbus.parse_ok()?,
                        JmcHsv57ModbusResponseType::InitAcceleration => self.modbus.parse_ok()?,
                        JmcHsv57ModbusResponseType::InitDeceleration => self.modbus.parse_ok()?,
                        JmcHsv57ModbusResponseType::SetSpeed => self.modbus.parse_ok()?,
                        JmcHsv57ModbusResponseType::GetSpeed => {
                            let mut result: Vec<u16, 1> = Vec::new();
                            self.modbus.parse_u16(&mut result)?;
                            let rpm_in_u16 = result.get(0).unwrap();
                            let rpm_in_i16 = u16_to_i16(*rpm_in_u16);
                            self.current_rpm = Some(rpm_in_i16);
                        }
                    };

                    return Poll::Pending;
                } else {
                    if !self.modbus_requests.is_empty() {
                        // setup next modbus request
                        match self.modbus_requests.pop_front().unwrap() {
                            JmcHsv57ModbusRequest::InitControlMode => {
                                // set P01-01 (0x0065) to 1
                                self.modbus_response_type =
                                    Some(JmcHsv57ModbusResponseType::InitControlMode);
                                self.modbus.set_holding(0x0065, 1)?;
                            }
                            JmcHsv57ModbusRequest::InitSpeedSource => {
                                // set P04-01 (0x0191) to 1
                                self.modbus_response_type =
                                    Some(JmcHsv57ModbusResponseType::InitSpeedSource);
                                self.modbus.set_holding(0x0191, 1)?;
                            }
                            JmcHsv57ModbusRequest::InitAcceleration { ms_per_1000_rpm } => {
                                // set P04-14 (0x019E) to unit 1ms/1000rpm
                                self.modbus_response_type =
                                    Some(JmcHsv57ModbusResponseType::InitAcceleration);
                                self.modbus.set_holding(0x019E, ms_per_1000_rpm)?;
                            }
                            JmcHsv57ModbusRequest::InitDeceleration { ms_per_1000_rpm } => {
                                // set P04-15 (0x019F) to unit 1ms/1000rpm
                                self.modbus_response_type =
                                    Some(JmcHsv57ModbusResponseType::InitDeceleration);
                                self.modbus.set_holding(0x019F, ms_per_1000_rpm)?;
                            }
                            JmcHsv57ModbusRequest::SetSpeed { rpm } => {
                                // set P04-02 (0x0192) to rpm (-6000 to 6000)
                                self.modbus_response_type =
                                    Some(JmcHsv57ModbusResponseType::SetSpeed);
                                self.modbus.set_holding(0x0192, rpm)?;
                            }
                            JmcHsv57ModbusRequest::GetSpeed => {
                                // get d08.F.SP (0x0842) for rpm
                                self.modbus_response_type =
                                    Some(JmcHsv57ModbusResponseType::GetSpeed);
                                self.modbus.get_holdings(0x0842, 1)?;
                            }
                        };

                        Poll::Pending
                    } else {
                        Poll::Ready(Ok(()))
                    }
                }
            }
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn desired_rpm(&mut self) -> i16 {
        match self.spindle_status {
            SpindleStatus::On { rpm } => rpm,
            SpindleStatus::Off => 0,
        }
    }
}

#[derive(Debug)]
pub enum SpindleDriverJmcHsv57Error<SerialTxError: Debug, SerialRxError: Debug> {
    ModbusSerial(ModbusSerialError<SerialTxError, SerialRxError>),
    QueueFull,
}

type SpindleDriverJmcHsv57ErrorAlias<Serial> =
    SpindleDriverJmcHsv57Error<<Serial as Write<u8>>::Error, <Serial as Read<u8>>::Error>;

impl<Serial> SpindleDriver for SpindleDriverJmcHsv57<Serial>
where
    Serial: Write<u8> + Read<u8>,
    <Serial as Write<u8>>::Error: Debug,
    <Serial as Read<u8>>::Error: Debug,
{
    type Error = SpindleDriverJmcHsv57ErrorAlias<Serial>;

    fn set(&mut self, status: SpindleStatus) {
        self.next_spindle_status = Some(status);
    }

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        match self.handle_modbus() {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(err)) => {
                return Poll::Ready(Err(SpindleDriverJmcHsv57Error::ModbusSerial(err)))
            }
            Poll::Ready(Ok(())) => {
                // pass through
            }
        }

        if !self.has_initialized {
            // initialize spindle over modbus
            self.modbus_requests
                .push_back(JmcHsv57ModbusRequest::InitControlMode)
                .map_err(|_| SpindleDriverJmcHsv57Error::QueueFull)?;

            self.modbus_requests
                .push_back(JmcHsv57ModbusRequest::InitSpeedSource)
                .map_err(|_| SpindleDriverJmcHsv57Error::QueueFull)?;

            self.modbus_requests
                .push_back(JmcHsv57ModbusRequest::InitAcceleration {
                    ms_per_1000_rpm: ACCELERATION_IN_MS_PER_1000_RPM,
                })
                .map_err(|_| SpindleDriverJmcHsv57Error::QueueFull)?;

            self.modbus_requests
                .push_back(JmcHsv57ModbusRequest::InitDeceleration {
                    ms_per_1000_rpm: ACCELERATION_IN_MS_PER_1000_RPM,
                })
                .map_err(|_| SpindleDriverJmcHsv57Error::QueueFull)?;

            self.has_initialized = true;

            return Poll::Pending;
        }

        if let Some(next_spindle_status) = self.next_spindle_status {
            if next_spindle_status != self.spindle_status {
                self.spindle_status = next_spindle_status;

                // set speed over modbus
                let rpm_in_i16 = self.desired_rpm();
                let rpm_in_u16 = i16_to_u16(rpm_in_i16);
                self.modbus_requests
                    .push_back(JmcHsv57ModbusRequest::SetSpeed { rpm: rpm_in_u16 })
                    .map_err(|_| SpindleDriverJmcHsv57Error::QueueFull)?;

                return Poll::Pending;
            }
        }

        // check if speed has been reached
        if let Some(current_rpm) = self.current_rpm {
            // if rpm within error bounds
            let desired_rpm = self.desired_rpm();
            if abs(current_rpm - desired_rpm) < RPM_ERROR_BOUND {
                return Poll::Ready(Ok(()));
            }
        }

        // get speed over modbus
        self.modbus_requests
            .push_back(JmcHsv57ModbusRequest::GetSpeed)
            .map_err(|_| SpindleDriverJmcHsv57Error::QueueFull)?;

        return Poll::Pending;
    }
}

pub struct SpindleDevice<Driver>
where
    Driver: SpindleDriver,
{
    driver: Driver,
}

impl<Driver> SpindleDevice<Driver>
where
    Driver: SpindleDriver,
{
    pub fn new(driver: Driver) -> Self {
        Self { driver }
    }
}

pub type SpindleError<Driver> = <Driver as SpindleDriver>::Error;

impl<Driver> Actuator for SpindleDevice<Driver>
where
    Driver: SpindleDriver,
{
    type Action = SpindleAction;
    type Error = Driver::Error;

    fn run(&mut self, action: &Self::Action) {
        match action {
            SpindleAction::Set { status } => self.driver.set(*status),
        }
    }

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        self.driver.poll()
    }
}
