// https://github.com/jfturcot/SimpleTimer
// https://github.com/khoih-prog/STM32_TimerInterrupt/blob/main/src/STM32_ISR_Timer-Impl.h
// https://playground.arduino.cc/Code/TimingRollover/

use core::sync::atomic::{AtomicU32, Ordering};
use defmt::Format;
use fugit::{TimerDurationU32 as TimerDuration, TimerInstantU32 as TimerInstant};
use fugit_timer::Timer;
use nb;

pub const TICK_TIMER_HZ: u32 = 1_000_000;
static TIME: AtomicU32 = AtomicU32::new(0);

pub fn setup<T>(timer: &mut T, max_ticks: u32) -> Result<(), T::Error>
where
    T: Timer<TICK_TIMER_HZ>,
{
    let max_duration = TimerDuration::<TICK_TIMER_HZ>::from_ticks(max_ticks);
    timer.start(max_duration)?;
    Ok(())
}

pub fn tick<T>(timer: &mut T, max_ticks: u32) -> Result<(), T::Error>
where
    T: Timer<TICK_TIMER_HZ>,
{
    let time_instant = timer.now();
    let time_ticks = time_instant.ticks();
    TIME.swap(time_ticks, Ordering::SeqCst);

    match timer.wait() {
        Ok(()) => {
            setup(timer, max_ticks)?;
        }
        Err(nb::Error::WouldBlock) => {}
        Err(nb::Error::Other(err)) => return Err(err),
    }

    Ok(())
}

pub fn now() -> TimerInstant<TICK_TIMER_HZ> {
    let time_ticks = TIME.load(Ordering::SeqCst);
    TimerInstant::from_ticks(time_ticks)
}

#[derive(Clone, Copy, Debug, Format)]
pub struct SubTimer {
    state: SubTimerState,
}

#[derive(Clone, Copy, Debug, Format)]
enum SubTimerState {
    Stop,
    Start {
        start: TimerInstant<TICK_TIMER_HZ>,
        duration: TimerDuration<TICK_TIMER_HZ>,
    },
}

impl SubTimer {
    pub fn new() -> Self {
        Self {
            state: SubTimerState::Stop,
        }
    }
}

#[derive(Debug)]
pub enum SubTimerError {
    NoStart,
}

impl Timer<TICK_TIMER_HZ> for SubTimer {
    type Error = SubTimerError;

    fn now(&mut self) -> TimerInstant<TICK_TIMER_HZ> {
        now()
    }

    fn start(&mut self, duration: TimerDuration<TICK_TIMER_HZ>) -> Result<(), Self::Error> {
        let now = self.now();

        self.state = SubTimerState::Start {
            start: now,
            duration,
        };

        Ok(())
    }

    fn cancel(&mut self) -> Result<(), Self::Error> {
        match self.state {
            SubTimerState::Stop => Err(SubTimerError::NoStart),
            SubTimerState::Start { .. } => {
                self.state = SubTimerState::Stop;

                Ok(())
            }
        }
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        let now = self.now();

        match self.state {
            SubTimerState::Stop => Err(nb::Error::Other(SubTimerError::NoStart)),
            SubTimerState::Start { start, duration } => {
                let duration_ticks = duration.ticks();
                let now_ticks = now.ticks();
                let start_ticks = start.ticks();

                // https://playground.arduino.cc/Code/TimingRollover/
                let wait_ticks = now_ticks.wrapping_sub(start_ticks);
                if wait_ticks > duration_ticks {
                    Ok(())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
        }
    }
}
