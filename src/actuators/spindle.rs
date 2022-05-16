// https://github.com/robert-budde/iHSV-Servo-Tool/blob/master/iHSV_Properties.py

use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use embedded_hal::serial::{Read, Write};
use fixedvec::{alloc_stack, FixedVec};
use heapless::Deque;
use num::abs;

use crate::actor::{ActorPoll, ActorReceive};
use crate::modbus::{ModbusSerial, ModbusSerialError, ModbusSerialErrorAlias};

#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum SpindleStatus {
    Off,
    On { rpm: i16 },
}

pub trait SpindleDriver {
    type Error;

    fn set(&mut self, status: SpindleStatus);
    fn poll(&mut self) -> Poll<Result<(), Self::Error>>;
}

#[derive(Clone, Copy, Debug, Format)]
enum JmcHsv57ModbusRequest {
    InitSpeedSource,
    SetSpeed { rpm: u16 },
    GetSpeed,
}

#[derive(Clone, Copy, Debug, Format)]
enum JmcHsv57ModbusResponseType {
    InitSpeedSource,
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

    pub fn handle_modbus(&mut self) -> Poll<Result<(), ModbusSerialErrorAlias<Serial>>> {
        match self.modbus.poll() {
            Poll::Ready(Ok(is_response_ready)) => {
                if is_response_ready {
                    // handle modbus response
                    match self.modbus_response_type.unwrap() {
                        JmcHsv57ModbusResponseType::InitSpeedSource => self.modbus.parse_ok()?,
                        JmcHsv57ModbusResponseType::SetSpeed => self.modbus.parse_ok()?,
                        JmcHsv57ModbusResponseType::GetSpeed => {
                            let mut result_space = alloc_stack!([u16; 1]);
                            let mut result = FixedVec::new(&mut result_space);
                            self.modbus.parse_u16(&mut result)?;
                            let rpm = result.get(0).unwrap();
                            self.set_current_rpm_from_u16(*rpm);
                        }
                    };

                    return Poll::Pending;
                } else {
                    if !self.modbus_requests.is_empty() {
                        // setup next modbus request
                        match self.modbus_requests.pop_front().unwrap() {
                            JmcHsv57ModbusRequest::InitSpeedSource => {
                                // set P04-01 (0x0191) to 1
                                self.modbus_response_type =
                                    Some(JmcHsv57ModbusResponseType::InitSpeedSource);
                                self.modbus.set_holding(0x0191, 1)?;
                            }
                            JmcHsv57ModbusRequest::SetSpeed { rpm } => {
                                // set P04-01 (0x0192) to rpm (-6000 to 6000)
                                self.modbus_response_type =
                                    Some(JmcHsv57ModbusResponseType::SetSpeed);
                                self.modbus.set_holding(0x0192, rpm)?;
                            }
                            JmcHsv57ModbusRequest::GetSpeed => {
                                // get d08.F.SP (0x0842) for rpm
                                self.modbus_response_type =
                                    Some(JmcHsv57ModbusResponseType::GetSpeed);
                                self.modbus.get_inputs(0x0842, 1)?;
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

    fn desired_rpm_as_u16(&mut self) -> u16 {
        let rpm = self.desired_rpm();
        // TODO two's complement
        rpm as u16
    }

    fn set_current_rpm_from_u16(&mut self, rpm: u16) {
        // TODO two's complement
        let rpm_as_i16 = rpm as i16;
        self.current_rpm = Some(rpm_as_i16);
    }
}

#[derive(Debug)]
pub enum SpindleDriverJmcHsv57Error<SerialTxError: Debug, SerialRxError: Debug> {
    ModbusSerial(ModbusSerialError<SerialTxError, SerialRxError>),
    QueueFull,
}

pub type SpindleDriverJmcHsv57ErrorAlias<Serial> =
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
                .push_back(JmcHsv57ModbusRequest::InitSpeedSource)
                .map_err(|_| SpindleDriverJmcHsv57Error::QueueFull)?;

            return Poll::Pending;
        }

        if let Some(next_spindle_status) = self.next_spindle_status {
            if next_spindle_status != self.spindle_status {
                // set speed over modbus
                let rpm = self.desired_rpm_as_u16();
                self.modbus_requests
                    .push_back(JmcHsv57ModbusRequest::SetSpeed { rpm })
                    .map_err(|_| SpindleDriverJmcHsv57Error::QueueFull)?;

                self.spindle_status = next_spindle_status;

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

pub struct Spindle<Driver>
where
    Driver: SpindleDriver,
{
    driver: Driver,
}

impl<Driver> Spindle<Driver>
where
    Driver: SpindleDriver,
{
    pub fn new(driver: Driver) -> Self {
        Spindle { driver }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct SpindleSetMessage {
    pub status: SpindleStatus,
}

impl<Driver> ActorReceive<SpindleSetMessage> for Spindle<Driver>
where
    Driver: SpindleDriver,
{
    fn receive(&mut self, action: &SpindleSetMessage) {
        self.driver.set(action.status)
    }
}

pub type SpindleError<Driver> = <Driver as SpindleDriver>::Error;

impl<Driver> ActorPoll for Spindle<Driver>
where
    Driver: SpindleDriver,
{
    type Error = Driver::Error;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        self.driver.poll()
    }
}
