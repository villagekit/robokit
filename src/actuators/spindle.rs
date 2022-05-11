use core::convert::Infallible;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::task::Poll;
use defmt::Format;
use embedded_hal::serial::{Read, Write};

use crate::actor::{ActorPoll, ActorReceive};

#[derive(Clone, Copy, Debug, Format)]
pub enum SpindleStatus {
    Off,
    On,
}

pub trait SpindleDriver {
    type Error;

    fn set(&mut self, status: SpindleStatus);
    fn poll(&mut self) -> Poll<Result<(), Self::Error>>;
}

pub struct SpindleDriverJmcHsv57<Word, SerialTx, SerialRx>
where
    SerialTx: Write<Word>,
    SerialRx: Read<Word>,
{
    word: PhantomData<Word>,
    tx: SerialTx,
    rx: SerialRx,
    has_initialized: bool,
    current_status: SpindleStatus,
    next_status: Option<SpindleStatus>,
}

impl<Word, SerialTx, SerialRx> SpindleDriverJmcHsv57<Word, SerialTx, SerialRx>
where
    SerialTx: Write<Word>,
    SerialRx: Read<Word>,
{
    pub fn new(tx: SerialTx, rx: SerialRx) -> Self {
        Self {
            word: PhantomData,
            tx,
            rx,
            has_initialized: false,
            current_status: SpindleStatus::Off,
            next_status: None,
        }
    }
}

impl<Word, SerialTx, SerialRx> SpindleDriver for SpindleDriverJmcHsv57<Word, SerialTx, SerialRx>
where
    SerialTx: Write<Word>,
    SerialRx: Read<Word>,
{
    type Error = Infallible;

    fn set(&mut self, status: SpindleStatus) {
        self.next_status = Some(status);
    }

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

pub struct Spindle<Driver>
where
    Driver: SpindleDriver,
{
    driver: Driver,
}

impl<Word, SerialTx, SerialRx> Spindle<SpindleDriverJmcHsv57<Word, SerialTx, SerialRx>>
where
    SerialTx: Write<Word>,
    SerialRx: Read<Word>,
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
