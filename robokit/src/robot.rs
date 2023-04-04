use core::task::Poll;

use crate::actuators::{axis::AxisAction, led::LedAction, spindle::SpindleAction};
use crate::error::Error;
use crate::runner::{ActuatorSet, Command, Runner};
use crate::scheduler::Scheduler;
use alloc::boxed::Box;
use heapless::Vec;

pub struct Robot<Leds, const LED_TIMER_HZ: u32, Axes, Spindles>
where
    Leds: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    Axes: ActuatorSet<Action = AxisAction>,
    Spindles: ActuatorSet<Action = SpindleAction>,
{
    scheduler: Scheduler<
        Command<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>,
        Runner<Leds, LED_TIMER_HZ, Axes, Spindles>,
    >,
}

impl<Leds, const LED_TIMER_HZ: u32, Axes, Spindles> Robot<Leds, LED_TIMER_HZ, Axes, Spindles>
where
    Leds: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    Axes: ActuatorSet<Action = AxisAction>,
    Spindles: ActuatorSet<Action = SpindleAction>,
{
    pub fn new(
        leds: Leds,
        axes: Axes,
        spindles: Spindles,
        run_commands: Vec<Command<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>, 32>,
        start_commands: Vec<Command<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>, 4>,
        stop_commands: Vec<Command<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>, 4>,
    ) -> Self {
        let runner = Runner::new(leds, axes, spindles);
        let scheduler = Scheduler::new(runner, run_commands, start_commands, stop_commands);

        Self { scheduler }
    }

    pub fn start(&mut self) {
        self.scheduler.start()
    }

    pub fn stop(&mut self) {
        self.scheduler.stop()
    }

    pub fn toggle(&mut self) {
        self.scheduler.toggle()
    }

    pub fn poll(&mut self) -> Poll<Result<(), Box<dyn Error>>> {
        self.scheduler.poll()
    }
}
