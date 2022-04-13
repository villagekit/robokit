use core::task::Poll;

use embedded_hal::digital::v2::OutputPin;
use fugit::MillisDurationU32 as MillisDuration;
use fugit_timer::Timer;

use crate::actor::{ActorPoll, ActorReceive};

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

#[derive(Debug)]
pub enum LedError<PinError, TimerError> {
    Pin(PinError),
    Timer(TimerError),
}

impl<P, T> ActorPoll for Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    type Error = LedError<P::Error, T::Error>;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        if let Some(state) = self.state {
            match state.status {
                LedBlinkStatus::Start => {
                    // start timer
                    self.timer
                        .start(state.duration)
                        .map_err(|err| LedError::Timer(err))?;

                    // turn led on
                    self.pin.set_high().map_err(|err| LedError::Pin(err))?;

                    // update state
                    self.state = Some(LedBlinkState {
                        status: LedBlinkStatus::Wait,
                        duration: state.duration,
                    });

                    Poll::Pending
                }
                LedBlinkStatus::Wait => match self.timer.wait() {
                    Err(nb::Error::Other(err)) => Poll::Ready(Err(LedError::Timer(err))),
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
                    self.timer.cancel().map_err(|err| LedError::Timer(err))?;

                    self.pin.set_low().map_err(|err| LedError::Pin(err))?;

                    self.state = None;

                    Poll::Ready(Ok(()))
                }
            }
        } else {
            Poll::Ready(Ok(()))
        }
    }
}
