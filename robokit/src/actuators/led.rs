use alloc::sync::Arc;
use async_trait::async_trait;
use core::future::Future;
use core::task::Poll;
use core::{cell::RefMut, fmt::Debug};
use defmt::Format;
use embedded_hal::digital::v2::{OutputPin, PinState};
use fugit::TimerDurationU32 as TimerDuration;
use fugit_timer::Timer;
use nb;

use crate::error::Error;

use super::Actuator;

#[derive(Clone, Copy, Debug, Format)]
pub enum LedAction<const TIMER_HZ: u32> {
    Set { is_on: bool },
    Blink { duration: TimerDuration<TIMER_HZ> },
}

pub trait AnyLed<const TIMER_HZ: u32>: Actuator<Action = LedAction<TIMER_HZ>> {}

#[derive(Clone, Copy, Debug, Format)]
pub enum LedBlinkStatus {
    Start,
    Wait,
    Done,
}

#[derive(Clone, Copy, Debug, Format)]
pub enum LedState<const TIMER_HZ: u32> {
    Set {
        is_on: bool,
    },
    Blink {
        status: LedBlinkStatus,
        duration: TimerDuration<TIMER_HZ>,
    },
}

#[derive(Clone, Copy, Debug, Format)]
pub struct LedDevice<P, T, const TIMER_HZ: u32>
where
    P: OutputPin,
    T: Timer<TIMER_HZ>,
{
    pin: P,
    timer: T,
    state: Option<LedState<TIMER_HZ>>,
}

impl<P, T, const TIMER_HZ: u32> AnyLed<TIMER_HZ> for LedDevice<P, T, TIMER_HZ>
where
    P: OutputPin,
    P::Error: Debug,
    T: Timer<TIMER_HZ>,
    T::Error: Debug,
{
}

impl<P, T, const TIMER_HZ: u32> LedDevice<P, T, TIMER_HZ>
where
    P: OutputPin,
    T: Timer<TIMER_HZ>,
{
    pub fn new(pin: P, timer: T) -> Self {
        Self {
            pin,
            timer,
            state: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LedError<PinError: Debug, TimerError: Debug> {
    PinSet(PinError),
    TimerStart(TimerError),
    TimerWait(TimerError),
}

impl<PinError: Debug, TimerError: Debug> Error for LedError<PinError, TimerError> {}

#[async_trait]
impl<P, T, const TIMER_HZ: u32> Actuator for LedDevice<P, T, TIMER_HZ>
where
    P: OutputPin,
    P::Error: Debug,
    T: Timer<TIMER_HZ>,
    T::Error: Debug,
{
    type Action = LedAction<TIMER_HZ>;
    type Error = LedError<P::Error, T::Error>;

    async fn run(&mut self, action: &Self::Action) -> {
        match action {
            LedAction::Set { is_on } => {
                self.state = Some(LedState::Set { is_on: *is_on });
            }
            LedAction::Blink { duration } => {
                self.state = Some(LedState::Blink {
                    status: LedBlinkStatus::Start,
                    duration: *duration,
                });
            }
        }

        LedDeviceFuture(Arc::new(RefMut::new(self)))
    }
}

pub struct LedDeviceFuture<'a, P, T, const TIMER_HZ: u32>(
    Arc<RefMut<'a, LedDevice<P, T, TIMER_HZ>>>,
)
where
    P: OutputPin,
    T: Timer<TIMER_HZ>;

impl<'a, P, T, const TIMER_HZ: u32> Future for LedDeviceFuture<'a, P, T, TIMER_HZ>
where
    P: OutputPin,
    P::Error: Debug,
    T: Timer<TIMER_HZ>,
    T::Error: Debug,
{
    type Output = Result<(), LedError<P::Error, T::Error>>;

    fn poll(&mut self) -> Poll<Self::Output> {
        match self.state {
            Some(LedState::Set { is_on }) => {
                // set led state
                self.pin
                    .set_state(PinState::from(is_on))
                    .map_err(LedError::PinSet)?;

                self.state = None;

                Poll::Ready(Ok(()))
            }
            Some(LedState::Blink { duration, status }) => {
                match status {
                    LedBlinkStatus::Start => {
                        // start timer
                        self.timer.start(duration).map_err(LedError::TimerStart)?;

                        // turn led on
                        self.pin.set_high().map_err(LedError::PinSet)?;

                        // update state
                        self.state = Some(LedState::Blink {
                            status: LedBlinkStatus::Wait,
                            duration,
                        });

                        Poll::Pending
                    }
                    LedBlinkStatus::Wait => match self.timer.wait() {
                        Err(nb::Error::Other(err)) => Poll::Ready(Err(LedError::TimerWait(err))),
                        Err(nb::Error::WouldBlock) => Poll::Pending,
                        Ok(()) => {
                            self.state = Some(LedState::Blink {
                                status: LedBlinkStatus::Done,
                                duration,
                            });

                            Poll::Pending
                        }
                    },
                    LedBlinkStatus::Done => {
                        self.pin.set_low().map_err(LedError::PinSet)?;

                        self.state = None;

                        Poll::Ready(Ok(()))
                    }
                }
            }
            None => Poll::Ready(Ok(())),
        }
    }
}
