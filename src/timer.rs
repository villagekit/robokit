// https://github.com/jfturcot/SimpleTimer
// https://github.com/khoih-prog/STM32_TimerInterrupt/blob/main/src/STM32_ISR_Timer-Impl.h
// https://playground.arduino.cc/Code/TimingRollover/

use core::cell::RefCell;
use defmt::Format;
use fugit::{TimerDurationU32 as TimerDuration, TimerInstantU32 as TimerInstant};
use fugit_timer::Timer;
use heapless::Vec;
use nb;

pub struct SuperTimer<T, const TIMER_HZ: u32>
where
    T: Timer<TIMER_HZ>,
{
    pub timer: T,
    pub timer_bits: u32,
    pub sub_timers: Vec<RefCell<SubTimer<TIMER_HZ>>, 16>,
}

impl<T, const TIMER_HZ: u32> SuperTimer<T, TIMER_HZ>
where
    T: Timer<TIMER_HZ>,
{
    pub fn sub(&mut self) -> &RefCell<SubTimer<TIMER_HZ>> {
        let now = self.timer.now();
        let sub_timer = SubTimer {
            now: None,
            state: SubTimerState::Stop,
        };
        let sub_timer_cell = RefCell::new(sub_timer);
        self.sub_timers.push(sub_timer_cell);
        &sub_timer_cell
    }

    pub fn start(&mut self) -> Result<(), T::Error> {
        self.timer.start(self.max_timer_duration())
    }

    pub fn cancel(&mut self) -> Result<(), T::Error> {
        self.timer.cancel()
    }

    pub fn tick(&mut self) -> nb::Result<(), T::Error> {
        match self.timer.wait() {
            Ok(()) => {
                defmt::panic!("SuperTimer overflow!");
            }
            Err(nb::Error::Other(err)) => Err(nb::Error::WouldBlock),
            Err(nb::Error::WouldBlock) => {
                self.tick_sub_timers(self.timer.now());
                self.cancel();
                self.start();

                Err(nb::Error::WouldBlock)
            }
        }
    }

    fn tick_sub_timers(&mut self, now: TimerInstant<TIMER_HZ>) {
        for sub_timer_cell in self.sub_timers.iter() {
            let mut sub_timer = sub_timer_cell.borrow_mut();
            sub_timer.tick(now)
        }
    }

    fn max_timer_duration(&mut self) -> TimerDuration<TIMER_HZ> {
        let max_duration_ticks = 2u32.pow(self.timer_bits);
        TimerDuration::<TIMER_HZ>::from_ticks(max_duration_ticks)
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct SubTimer<const TIMER_HZ: u32> {
    pub now: Option<TimerInstant<TIMER_HZ>>,
    pub state: SubTimerState<TIMER_HZ>,
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
    pub fn tick(&mut self, now: TimerInstant<TIMER_HZ>) {
        self.now = Some(now);
    }
}

#[derive(Debug)]
pub enum SubTimerError {
    NoTick,
    NoStart,
}

impl<const TIMER_HZ: u32> Timer<TIMER_HZ> for SubTimer<TIMER_HZ> {
    type Error = SubTimerError;

    fn now(&mut self) -> TimerInstant<TIMER_HZ> {
        self.now.unwrap()
    }

    fn start(&mut self, duration: TimerDuration<TIMER_HZ>) -> Result<(), Self::Error> {
        if let Some(now) = self.now {
            self.state = SubTimerState::Start {
                start: now,
                duration,
            };

            Ok(())
        } else {
            Err(SubTimerError::NoTick)
        }
    }

    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.state = SubTimerState::Stop;

        Ok(())
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        if let Some(now) = self.now {
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
        } else {
            Err(nb::Error::Other(SubTimerError::NoTick))
        }
    }
}
