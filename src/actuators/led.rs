use core::task::Poll;

use embedded_hal::digital::v2::OutputPin;
use fugit::MillisDurationU32 as MillisDuration;
use fugit_timer::Timer;

use crate::actor::{ActorPoll, ActorReceive};
use crate::error::Error;

#[derive(Clone, Copy)]
pub enum LedBlinkStatus {
    Start,
    Wait,
    Done,
}

#[derive(Clone, Copy)]
pub struct LedBlinkState {
    status: LedBlinkStatus,
    duration: MillisDuration,
}

pub struct Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pin: P,
    timer: T,
    state: Option<LedBlinkState>,
}

impl<P, T> Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pub fn new(pin: P, timer: T) -> Self {
        Led {
            pin,
            timer,
            state: None,
        }
    }
}

pub struct LedBlinkMessage {
    pub duration: MillisDuration,
}

impl<P, T> ActorReceive for Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    type Message = LedBlinkMessage;

    fn receive(&mut self, action: &Self::Message) {
        self.state = Some(LedBlinkState {
            status: LedBlinkStatus::Start,
            duration: action.duration,
        });
    }
}

impl<P, T> ActorPoll for Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    fn poll(&mut self) -> Poll<Result<(), Error>> {
        if let Some(state) = self.state {
            match state.status {
                LedBlinkStatus::Start => {
                    // start timer
                    self.timer
                        .start(state.duration)
                        .map_err(|_err| Error::Timer)?;

                    // turn led on
                    self.pin.set_high().map_err(|_err| Error::Pin)?;

                    // update state
                    self.state = Some(LedBlinkState {
                        status: LedBlinkStatus::Wait,
                        duration: state.duration,
                    });

                    Poll::Pending
                }
                LedBlinkStatus::Wait => match self.timer.wait() {
                    Err(nb::Error::Other(_err)) => Poll::Ready(Err(Error::Timer)),
                    Err(nb::Error::WouldBlock) => Poll::Pending,
                    Ok(()) => {
                        self.state = Some(LedBlinkState {
                            status: LedBlinkStatus::Done,
                            duration: state.duration,
                        });

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
        } else {
            Poll::Ready(Err(Error::Other))
        }
    }
}
