// https://github.com/robert-budde/iHSV-Servo-Tool/blob/master/iHSV_Properties.py

use core::convert::Infallible;
use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use embedded_hal::serial::{Read, Write};
use rmodbus::ErrorKind as ModbusError;

use crate::actor::{ActorPoll, ActorReceive};
use crate::modbus::{ModbusSerial, ModbusSerialError};

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

pub struct SpindleDriverJmcHsv57<'a, SerialTx, SerialRx>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    modbus: ModbusSerial<'a, SerialTx, SerialRx>,
    has_initialized: bool,
    spindle_status: SpindleStatus,
    next_spindle_status: Option<SpindleStatus>,
}

impl<'a, SerialTx, SerialRx> SpindleDriverJmcHsv57<'a, SerialTx, SerialRx>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    pub fn new(
        tx: SerialTx,
        rx: SerialRx,
        request_bytes_space: &'a mut [u8],
        response_bytes_space: &'a mut [u8],
    ) -> Self {
        Self {
            modbus: ModbusSerial::new(tx, rx, 1, request_bytes_space, response_bytes_space),
            has_initialized: false,
            spindle_status: SpindleStatus::Off,
            next_spindle_status: None,
        }
    }
}

pub enum SpindleDriverJmcHsv57Error<SerialTx, SerialRx>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    ModbusSerial(ModbusSerialError<SerialTx, SerialRx>),
}

impl<'a, SerialTx, SerialRx> SpindleDriver for SpindleDriverJmcHsv57<'a, SerialTx, SerialRx>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    type Error = SpindleDriverJmcHsv57Error<SerialTx, SerialRx>;

    fn set(&mut self, status: SpindleStatus) {
        self.next_spindle_status = Some(status);
    }

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        match self.modbus.poll() {
            Poll::Ready(Ok(is_response_ready)) => {
                // handle modbus response
                return Poll::Pending;
            }
            Poll::Ready(Err(err)) => {
                return Poll::Ready(Err(SpindleDriverJmcHsv57Error::ModbusSerial(err)))
            }
            Poll::Pending => {}
        }

        if !self.has_initialized {
            // initialize spindle over modbus
            return Poll::Pending;
            // set P04-01 (0x0191) to 1
        }

        if let Some(next_spindle_status) = self.next_spindle_status {
            if next_spindle_status != self.spindle_status {
                // set speed over modbus
                return Poll::Pending;
                // set P04-01 (0x0192) to rpm (-6000 to 6000)
            }
        }

        // check over modbus if speed has been reached

        Poll::Pending
    }
}

pub struct Spindle<Driver>
where
    Driver: SpindleDriver,
{
    driver: Driver,
}

impl<'a, SerialTx, SerialRx> Spindle<SpindleDriverJmcHsv57<'a, SerialTx, SerialRx>>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    pub fn new_jmchsv57(
        tx: SerialTx,
        rx: SerialRx,
        request_bytes_space: &'a mut [u8],
        response_bytes_space: &'a mut [u8],
    ) -> Self {
        let driver = SpindleDriverJmcHsv57::new(tx, rx, request_bytes_space, response_bytes_space);

        Spindle { driver }
    }
}

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
    type Error = SpindleError<Driver>;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        self.driver.poll()
    }
}
