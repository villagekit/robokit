// https://github.com/robert-budde/iHSV-Servo-Tool/blob/master/iHSV_Properties.py

use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use embedded_hal::serial::{Read, Write};
use heapless::Deque;
use num::abs;

use crate::actor::{ActorPoll, ActorReceive};
use crate::modbus::{ModbusSerial, ModbusSerialError, ModbusSerialErrorAlias};

#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum SpindleStatus {
    Off,
    On,
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
    desired_rpm: u16,
    current_rpm: Option<u16>,
}

const RPM_ERROR_BOUND: u16 = 2;

impl<Serial> SpindleDriverJmcHsv57<Serial>
where
    Serial: Write<u8> + Read<u8>,
    <Serial as Write<u8>>::Error: Debug,
    <Serial as Read<u8>>::Error: Debug,
{
    pub fn new(serial: Serial, desired_rpm: u16) -> Self {
        Self {
            modbus: ModbusSerial::new(serial, 1),
            modbus_requests: Deque::new(),
            modbus_response_type: None,
            has_initialized: false,
            spindle_status: SpindleStatus::Off,
            next_spindle_status: None,
            desired_rpm,
            current_rpm: None,
        }
    }

    pub fn handle_modbus(&mut self) -> Poll<Result<(), ModbusSerialErrorAlias<Serial>>> {
        match self.modbus.poll() {
            Poll::Ready(Ok(is_response_ready)) => {
                if is_response_ready {
                    // handle modbus response
                    match self.modbus_response_type.unwrap() {
                        JmcHsv57ModbusResponseType::InitSpeedSource => {
                            // TODO
                        }
                        JmcHsv57ModbusResponseType::SetSpeed => {
                            // TODO
                        }
                        JmcHsv57ModbusResponseType::GetSpeed => {
                            // TODO
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
                self.modbus_requests
                    .push_back(JmcHsv57ModbusRequest::SetSpeed {
                        rpm: self.desired_rpm,
                    })
                    .map_err(|_| SpindleDriverJmcHsv57Error::QueueFull)?;

                return Poll::Pending;
            }
        }

        // check if speed has been reached
        if let Some(current_rpm) = self.current_rpm {
            // if rpm within error bounds
            if abs((current_rpm as i16) - (self.desired_rpm as i16)) < (RPM_ERROR_BOUND as i16) {
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
    status: SpindleStatus,
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
