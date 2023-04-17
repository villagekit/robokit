use core::task::Poll;
use heapless::Vec;

use crate::actuators::{axis::AxisAction, led::LedAction, spindle::SpindleAction, ActuatorSet};
use crate::runner::{Command, Runner, RunnerError};
use crate::scheduler::Scheduler;

pub struct Robot<
    const LED_TIMER_HZ: u32,
    const RUN_COMMANDS_COUNT: usize,
    const START_COMMANDS_COUNT: usize,
    const STOP_COMMANDS_COUNT: usize,
    const ACTIVE_COMMANDS_COUNT: usize,
    LedSet,
    AxisSet,
    SpindleSet,
> where
    LedSet: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    AxisSet: ActuatorSet<Action = AxisAction>,
    SpindleSet: ActuatorSet<Action = SpindleAction>,
{
    scheduler: Scheduler<
        Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>,
        Runner<LED_TIMER_HZ, ACTIVE_COMMANDS_COUNT, LedSet, AxisSet, SpindleSet>,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
    >,
}

impl<
        const LED_TIMER_HZ: u32,
        const RUN_COMMANDS_COUNT: usize,
        const START_COMMANDS_COUNT: usize,
        const STOP_COMMANDS_COUNT: usize,
        const ACTIVE_COMMANDS_COUNT: usize,
        LedSet,
        AxisSet,
        SpindleSet,
    >
    Robot<
        LED_TIMER_HZ,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
        ACTIVE_COMMANDS_COUNT,
        LedSet,
        AxisSet,
        SpindleSet,
    >
where
    LedSet: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    AxisSet: ActuatorSet<Action = AxisAction>,
    SpindleSet: ActuatorSet<Action = SpindleAction>,
{
    pub fn new(
        run_commands: Vec<
            Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>,
            RUN_COMMANDS_COUNT,
        >,
        start_commands: Vec<
            Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>,
            START_COMMANDS_COUNT,
        >,
        stop_commands: Vec<
            Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>,
            STOP_COMMANDS_COUNT,
        >,
        leds: LedSet,
        axes: AxisSet,
        spindles: SpindleSet,
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

    pub fn poll(
        &mut self,
    ) -> Poll<
        Result<
            (),
            RunnerError<
                LedSet::Id,
                LedSet::Error,
                AxisSet::Id,
                AxisSet::Error,
                SpindleSet::Id,
                SpindleSet::Error,
            >,
        >,
    > {
        self.scheduler.poll()
    }
}
