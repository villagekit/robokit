use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use fugit::TimerDurationU32 as TimerDuration;
use fugit_timer::Timer;

use crate::actor::{ActorPoll, ActorReceive};

#[derive(Clone, Copy)]
pub enum LedBlinkStatus {
    Start,
    Wait,
    Done,
}

#[derive(Clone, Copy)]
pub struct LedBlinkState<const FREQ: u32> {
    status: LedBlinkStatus,
    duration: TimerDuration<FREQ>,
}

pub struct Led<P, T, const FREQ: u32>
where
    P: OutputPin,
    T: Timer<FREQ>,
{
    pin: P,
    timer: T,
    state: Option<LedBlinkState<FREQ>>,
}

impl<P, T, const FREQ: u32> Led<P, T, FREQ>
where
    P: OutputPin,
    T: Timer<FREQ>,
{
    pub fn new(pin: P, timer: T) -> Self {
        Led {
            pin,
            timer,
            state: None,
        }
    }
}

#[derive(Format)]
pub struct LedBlinkMessage<const FREQ: u32> {
    pub duration: TimerDuration<FREQ>,
}

impl<P, T, const FREQ: u32> ActorReceive for Led<P, T, FREQ>
where
    P: OutputPin,
    T: Timer<FREQ>,
{
    type Message = LedBlinkMessage<FREQ>;

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

impl<P, T, const FREQ: u32> ActorPoll for Led<P, T, FREQ>
where
    P: OutputPin,
    T: Timer<FREQ>,
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
