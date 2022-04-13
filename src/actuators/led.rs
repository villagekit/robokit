use core::task::Poll;

use embedded_hal::digital::v2::OutputPin;
use fugit::MillisDurationU32 as MillisDuration;
use fugit_timer::Timer;

use crate::actor::{ActorFuture, ActorReceive};
use crate::error::Error;

pub struct Led<'a, P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pin: P,
    timer: T,
    future: Option<LedBlinkFuture<'a, P, T>>,
}

impl<'a, P, T> Led<'a, P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pub fn new(pin: P, timer: T) -> Self {
        Led {
            pin,
            timer,
            future: None,
        }
    }
}

pub struct LedBlinkMessage {
    pub duration: MillisDuration,
}

impl<'a, P, T> ActorReceive<LedBlinkMessage> for Led<'a, P, T>
where
    P: 'a + OutputPin,
    T: 'a + Timer<1_000>,
{
    fn receive(&'a mut self, action: &LedBlinkMessage) {
        self.future = Some(LedBlinkFuture {
            pin: &mut self.pin,
            timer: &mut self.timer,
            status: LedBlinkStatus::Start,
            duration: action.duration,
        })
    }
}

impl<'a, P, T> ActorFuture for Led<'a, P, T>
where
    P: 'a + OutputPin,
    T: 'a + Timer<1_000>,
{
    fn poll(&'a mut self) -> Poll<Result<(), Error>> {
        self.future.poll()
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

impl<'a, P, T> ActorFuture for LedBlinkFuture<'a, P, T>
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
