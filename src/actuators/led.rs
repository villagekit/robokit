use core::task::Poll;

use embedded_hal::digital::v2::OutputPin;
use fugit::MillisDurationU32 as MillisDuration;
use fugit_timer::Timer;

use crate::actuator::{Actuator, Future};
use crate::error::Error;

pub struct Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pin: P,
    timer: T,
}

impl<P, T> Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pub fn new(pin: P, timer: T) -> Self {
        Led { pin, timer }
    }
}

pub struct LedBlinkMessage {
    pub duration: MillisDuration,
}

impl<'a, P, T> Actuator<'a> for Led<P, T>
where
    P: 'a + OutputPin,
    T: 'a + Timer<1_000>,
{
    type Message = LedBlinkMessage;
    type Future = LedBlinkFuture<'a, P, T>;

    fn command(&'a mut self, action: &Self::Message) -> Self::Future {
        LedBlinkFuture {
            pin: &mut self.pin,
            timer: &mut self.timer,
            status: LedBlinkStatus::Start,
            duration: action.duration,
        }
    }
}

pub enum LedBlinkStatus {
    Start,
    Wait,
    Done,
}

pub struct LedBlinkFuture<'a, P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pin: &'a mut P,
    timer: &'a mut T,
    status: LedBlinkStatus,
    duration: MillisDuration,
}

impl<'a, P, T> Future for LedBlinkFuture<'a, P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    fn poll(&mut self) -> Poll<Result<(), Error>> {
        // TODO handle errors
        match self.status {
            LedBlinkStatus::Start => {
                // start timer
                self.timer
                    .start(self.duration)
                    .map_err(|_err| Error::Timer)?;

                // turn led on
                self.pin.set_high().map_err(|_err| Error::Pin)?;

                // update status
                self.status = LedBlinkStatus::Wait;

                Poll::Pending
            }
            LedBlinkStatus::Wait => match self.timer.wait() {
                Err(nb::Error::Other(_err)) => Poll::Ready(Err(Error::Timer)),
                Err(nb::Error::WouldBlock) => Poll::Pending,
                Ok(()) => {
                    self.status = LedBlinkStatus::Done;

                    Poll::Pending
                }
            },
            LedBlinkStatus::Done => {
                // if the timer isn't cancelled, it's periodic
                // and will automatically return on next call.
                self.timer.cancel().map_err(|_err| Error::Timer)?;

                self.pin.set_low().map_err(|_err| Error::Pin)?;

                Poll::Ready(Ok(()))
            }
        }
    }
}