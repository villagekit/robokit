use alloc::boxed::Box;
use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use fugit::TimerDurationU32 as TimerDuration;
use fugit_timer::Timer;

use super::Actuator;

#[derive(Clone, Copy, Debug, Format)]
pub enum LedAction<const TIMER_HZ: u32> {
    Blink { duration: TimerDuration<TIMER_HZ> },
}

pub trait AnyLedError: Debug {}
pub trait AnyLed<const TIMER_HZ: u32>:
    Actuator<LedAction<TIMER_HZ>, Error = Box<dyn AnyLedError>>
{
}

#[derive(Clone, Copy, Debug, Format)]
pub enum LedBlinkStatus {
    Start,
    Wait,
    Done,
}

#[derive(Clone, Copy, Debug, Format)]
pub struct LedBlinkState<const TIMER_HZ: u32> {
    status: LedBlinkStatus,
    duration: TimerDuration<TIMER_HZ>,
}

#[derive(Clone, Copy, Debug, Format)]
pub struct LedDevice<P, T, const TIMER_HZ: u32>
where
    P: OutputPin,
    T: Timer<TIMER_HZ>,
{
    pin: P,
    timer: T,
    state: Option<LedBlinkState<TIMER_HZ>>,
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
    Pin(PinError),
    Timer(TimerError),
}

impl<PinError: Debug, TimerError: Debug> AnyLedError for LedError<PinError, TimerError> {}

impl<P, T, const TIMER_HZ: u32> Actuator<LedAction<TIMER_HZ>> for LedDevice<P, T, TIMER_HZ>
where
    P: OutputPin,
    P::Error: Debug,
    T: Timer<TIMER_HZ>,
    T::Error: Debug,
{
    type Error = Box<dyn AnyLedError>;

    fn receive(&mut self, action: &LedAction<TIMER_HZ>) {
        match action {
            LedAction::Blink { duration } => {
                self.state = Some(LedBlinkState {
                    status: LedBlinkStatus::Start,
                    duration: *duration,
                });
            }
        }
    }

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
