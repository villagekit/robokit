// https://github.com/jfturcot/SimpleTimer
// https://github.com/khoih-prog/STM32_TimerInterrupt/blob/main/src/STM32_ISR_Timer-Impl.h
// https://playground.arduino.cc/Code/TimingRollover/

use alloc::rc::Rc;
use core::sync::atomic::{AtomicU32, Ordering};
use defmt::Format;
use fugit::{TimerDurationU32 as TimerDuration, TimerInstantU32 as TimerInstant};
use fugit_timer::Timer;
use nb;

pub struct SuperTimer<T, const TIMER_HZ: u32>
where
    T: Timer<TIMER_HZ>,
{
    now: Rc<AtomicU32>,
    timer: T,
    max_ticks: u32,
}

impl<T, const TIMER_HZ: u32> SuperTimer<T, TIMER_HZ>
where
    T: Timer<TIMER_HZ>,
{
    pub fn new(timer: T, max_ticks: u32) -> Self {
        Self {
            now: Rc::new(AtomicU32::new(0)),
            timer,
            max_ticks,
        }
    }
    pub fn setup(&mut self) -> Result<(), T::Error> {
        let max_duration = TimerDuration::<TIMER_HZ>::from_ticks(self.max_ticks);
        self.timer.start(max_duration)?;
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), T::Error> {
        let time_instant = self.timer.now();
        let time_ticks = time_instant.ticks();
        self.now.swap(time_ticks, Ordering::SeqCst);

        match self.timer.wait() {
            Ok(()) => {
                self.setup()?;
            }
            Err(nb::Error::WouldBlock) => {}
            Err(nb::Error::Other(err)) => return Err(err),
        }

        Ok(())
    }

    pub fn now(&self) -> TimerInstant<TIMER_HZ> {
        let time_ticks = self.now.load(Ordering::SeqCst);
        TimerInstant::from_ticks(time_ticks)
    }

    pub fn sub(&self) -> SubTimer<TIMER_HZ> {
        SubTimer::new(self.now.clone())
    }
}

#[derive(Clone)]
pub struct SubTimer<const TIMER_HZ: u32> {
    now: Rc<AtomicU32>,
    state: SubTimerState<TIMER_HZ>,
}

#[derive(Clone, Copy, Debug, Format)]
enum SubTimerState<const TIMER_HZ: u32> {
    Stop,
    Start {
        start: TimerInstant<TIMER_HZ>,
        duration: TimerDuration<TIMER_HZ>,
    },
}

impl<const TIMER_HZ: u32> SubTimer<TIMER_HZ> {
    pub fn new(now: Rc<AtomicU32>) -> Self {
        Self {
            now,
            state: SubTimerState::Stop,
        }
    }
}

#[derive(Debug)]
pub enum SubTimerError {
    NoStart,
}

impl<const TIMER_HZ: u32> Timer<TIMER_HZ> for SubTimer<TIMER_HZ> {
    type Error = SubTimerError;

    fn now(&mut self) -> TimerInstant<TIMER_HZ> {
        let time_ticks = self.now.load(Ordering::SeqCst);
        TimerInstant::from_ticks(time_ticks)
    }

    fn start(&mut self, duration: TimerDuration<TIMER_HZ>) -> Result<(), Self::Error> {
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
