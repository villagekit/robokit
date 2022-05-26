// https://github.com/jfturcot/SimpleTimer
// https://github.com/khoih-prog/STM32_TimerInterrupt/blob/main/src/STM32_ISR_Timer-Impl.h
// https://playground.arduino.cc/Code/TimingRollover/

use core::sync::atomic::{AtomicU32, Ordering};
use defmt::Format;
use fugit::{TimerDurationU32 as TimerDuration, TimerInstantU32 as TimerInstant};
use fugit_timer::Timer;
use nb;

static TIME: AtomicU32 = AtomicU32::new(0);

pub fn tick<T, const TIMER_HZ: u32>(timer: &mut T, max_ticks: u32) -> Result<(), T::Error>
where
    T: Timer<TIMER_HZ>,
{
    let time_instant = timer.now();
    let time_ticks = time_instant.ticks();
    TIME.swap(time_ticks, Ordering::SeqCst);

    match timer.wait() {
        Ok(()) => {
            let max_duration = TimerDuration::<TIMER_HZ>::from_ticks(max_ticks);
            timer.start(max_duration)?;
        }
        Err(nb::Error::WouldBlock) => {}
        Err(nb::Error::Other(err)) => return Err(err),
    }

    Ok(())
}

pub fn now<const TIMER_HZ: u32>() -> TimerInstant<TIMER_HZ> {
    let time_ticks = TIME.load(Ordering::SeqCst);
    TimerInstant::<TIMER_HZ>::from_ticks(time_ticks)
}

#[derive(Clone, Copy, Debug, Format)]
pub struct SubTimer<const TIMER_HZ: u32> {
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

impl<const TIMER_HZ: u32> Timer<TIMER_HZ> for SubTimer<TIMER_HZ> {
    type Error = SubTimerError;

    fn now(&mut self) -> TimerInstant<TIMER_HZ> {
        now()
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
        self.state = SubTimerState::Stop;

        Ok(())
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        let now = self.now();

        if let SubTimerState::Start { start, duration } = self.state {
            let wait_duration = now - start;
            if wait_duration < duration {
                Err(nb::Error::WouldBlock)
            } else {
                Ok(())
            }
        } else {
            Err(nb::Error::Other(SubTimerError::NoStart))
        }
    }
}
