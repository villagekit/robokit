use anyhow::{anyhow, Context, Error};
use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use fugit::TimerDurationU32 as TimerDuration;
use fugit_timer::Timer;

use crate::actor::{ActorPoll, ActorReceive};

pub trait Led<const TIMER_HZ: u32>: ActorReceive<LedBlinkMessage<TIMER_HZ>> + ActorPoll {}

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

#[derive(Clone, Copy, Debug, Format)]
pub struct LedBlinkMessage<const TIMER_HZ: u32> {
    pub duration: TimerDuration<TIMER_HZ>,
}

impl<P, T, const TIMER_HZ: u32> ActorReceive<LedBlinkMessage<TIMER_HZ>>
    for LedDevice<P, T, TIMER_HZ>
where
    P: OutputPin,
    T: Timer<TIMER_HZ>,
{
    fn receive(&mut self, action: &LedBlinkMessage<TIMER_HZ>) {
        self.state = Some(LedBlinkState {
            status: LedBlinkStatus::Start,
            duration: action.duration,
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LedError<PinError: Debug, TimerError: Debug> {
    Pin(PinError),
    Timer(TimerError),
}

impl<P, T, const TIMER_HZ: u32> ActorPoll for LedDevice<P, T, TIMER_HZ>
where
    P: OutputPin,
    P::Error: Debug,
    T: Timer<TIMER_HZ>,
    T::Error: Debug,
{
    fn poll(&mut self) -> Poll<Result<(), Error>> {
        if let Some(state) = self.state {
            match state.status {
                LedBlinkStatus::Start => {
                    // start timer
                    self.timer
                        .start(state.duration)
                        .context("Failed to start timer.")
                        .map_err(Error::msg)?;

                    // turn led on
                    self.pin
                        .set_high()
                        .context("Failed to set pin high.")
                        .map_err(Error::msg)?;

                    // update state
                    self.state = Some(LedBlinkState {
                        status: LedBlinkStatus::Wait,
                        duration: state.duration,
                    });

                    Poll::Pending
                }
                LedBlinkStatus::Wait => match self.timer.wait() {
                    Err(nb::Error::Other(err)) => Poll::Ready(anyhow!(err)),
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
                    self.pin
                        .set_low()
                        .context("Failed to set pin low.")
                        .map_err(Error::msg)?;

                    self.state = None;

                    Poll::Ready(Ok(()))
                }
            }
        } else {
            Poll::Ready(Ok(()))
        }
    }
}
