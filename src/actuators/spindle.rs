use core::convert::Infallible;
use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use embedded_hal::serial::{Read, Write};
use heapless::Deque;
use nb;
use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};

use crate::actor::{ActorPoll, ActorReceive};

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

enum SpindleSerialStatus {
    Idle,
    Writing,
    Reading,
}

pub struct SpindleDriverJmcHsv57<SerialTx, SerialRx>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    tx: SerialTx,
    rx: SerialRx,
    has_initialized: bool,
    spindle_status: SpindleStatus,
    next_spindle_status: Option<SpindleStatus>,
    serial_status: SpindleSerialStatus,
    current_serial_request: Option<Deque<u8, 256>>,
    current_serial_response: Option<Deque<u8, 256>>,
}

impl<SerialTx, SerialRx> SpindleDriverJmcHsv57<SerialTx, SerialRx>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    pub fn new(tx: SerialTx, rx: SerialRx) -> Self {
        Self {
            tx,
            rx,
            has_initialized: false,
            spindle_status: SpindleStatus::Off,
            next_spindle_status: None,
            serial_status: SpindleSerialStatus::Idle,
            current_serial_request: None,
            current_serial_response: None,
        }
    }
}

pub enum SpindleDriverJmcHsv57Error<SerialTx, SerialRx>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    SerialTx(SerialTx::Error),
    SerialRx(SerialRx::Error),
    Unexpected,
}

impl<SerialTx, SerialRx> SpindleDriver for SpindleDriverJmcHsv57<SerialTx, SerialRx>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    type Error = SpindleDriverJmcHsv57Error<SerialTx, SerialRx>;

    fn set(&mut self, status: SpindleStatus) {
        self.next_spindle_status = Some(status);
    }

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        if let SpindleSerialStatus::Writing = self.serial_status {
            if let Some(current_serial_request) = self.current_serial_request.as_mut() {
                if let Some(next_write) = current_serial_request.pop_front() {
                    match self.tx.write(next_write) {
                        Ok(()) => {}
                        Err(nb::Error::WouldBlock) => {}
                        Err(nb::Error::Other(err)) => {
                            return Poll::Ready(Err(SpindleDriverJmcHsv57Error::SerialTx(err)))
                        }
                    }
                } else {
                    match self.tx.flush() {
                        Ok(()) => {
                            self.serial_status = SpindleSerialStatus::Reading;
                        }
                        Err(nb::Error::WouldBlock) => {}
                        Err(nb::Error::Other(err)) => {
                            return Poll::Ready(Err(SpindleDriverJmcHsv57Error::SerialTx(err)))
                        }
                    }
                }
                return Poll::Pending;
            } else {
                return Poll::Ready(Err(SpindleDriverJmcHsv57Error::Unexpected));
            }
        }

        if !self.has_initialized {
            // initialize spindle over modbus
            return Poll::Pending;
        }

        if let Some(next_spindle_status) = self.next_spindle_status {
            if next_spindle_status != self.spindle_status {
                // set speed over modbus
                return Poll::Pending;
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

impl<SerialTx, SerialRx> Spindle<SpindleDriverJmcHsv57<SerialTx, SerialRx>>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    pub fn new_jmchsv57(tx: SerialTx, rx: SerialRx) -> Self {
        let driver = SpindleDriverJmcHsv57::new(tx, rx);

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
