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
        let runner: Runner<LED_TIMER_HZ, ACTIVE_COMMANDS_COUNT, _, _, _> =
            Runner::new(leds, axes, spindles);
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

    pub fn builder() -> RobotBuilder1<(), (), ()> {
        RobotBuilder1::new()
    }
}

pub struct RobotBuilder1<LedSet, AxisSet, SpindleSet> {
    leds: LedSet,
    axes: AxisSet,
    spindles: SpindleSet,
}

impl RobotBuilder1<(), (), ()> {
    pub fn new() -> Self {
        Self {
            leds: (),
            axes: (),
            spindles: (),
        }
    }
}

impl<AxisSet, SpindleSet> RobotBuilder1<(), AxisSet, SpindleSet> {
    pub fn with_leds<const LED_TIMER_HZ: u32, LedSet>(
        self,
        leds: LedSet,
    ) -> RobotBuilder1<LedSet, AxisSet, SpindleSet>
    where
        LedSet: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    {
        RobotBuilder1 {
            leds,
            axes: self.axes,
            spindles: self.spindles,
        }
    }
}

impl<LedSet, SpindleSet> RobotBuilder1<LedSet, (), SpindleSet> {
    pub fn with_axes<AxisSet>(self, axes: AxisSet) -> RobotBuilder1<LedSet, AxisSet, SpindleSet>
    where
        AxisSet: ActuatorSet<Action = AxisAction>,
    {
        RobotBuilder1 {
            leds: self.leds,
            axes,
            spindles: self.spindles,
        }
    }
}

impl<LedSet, AxisSet> RobotBuilder1<LedSet, AxisSet, ()> {
    pub fn with_spindles<SpindleSet>(
        self,
        spindles: SpindleSet,
    ) -> RobotBuilder1<LedSet, AxisSet, SpindleSet>
    where
        SpindleSet: ActuatorSet<Action = SpindleAction>,
    {
        RobotBuilder1 {
            leds: self.leds,
            axes: self.axes,
            spindles,
        }
    }
}

impl<const LED_TIMER_HZ: u32, LedSet, AxisSet, SpindleSet>
    RobotBuilder1<LedSet, AxisSet, SpindleSet>
where
    LedSet: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    AxisSet: ActuatorSet<Action = AxisAction>,
    SpindleSet: ActuatorSet<Action = SpindleAction>,
{
    pub fn build<
        const RUN_COMMANDS_COUNT: usize,
        const START_COMMANDS_COUNT: usize,
        const STOP_COMMANDS_COUNT: usize,
    >(
        self,
    ) -> RobotBuilder2<
        LED_TIMER_HZ,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
        LedSet,
        AxisSet,
        SpindleSet,
    > {
        RobotBuilder2::new(self)
    }
}

pub struct RobotBuilder2<
    const LED_TIMER_HZ: u32,
    const RUN_COMMANDS_COUNT: usize,
    const START_COMMANDS_COUNT: usize,
    const STOP_COMMANDS_COUNT: usize,
    LedSet,
    AxisSet,
    SpindleSet,
> where
    LedSet: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    AxisSet: ActuatorSet<Action = AxisAction>,
    SpindleSet: ActuatorSet<Action = SpindleAction>,
{
    leds: LedSet,
    axes: AxisSet,
    spindles: SpindleSet,
    run_commands:
        Vec<Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>, RUN_COMMANDS_COUNT>,
    start_commands:
        Vec<Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>, START_COMMANDS_COUNT>,
    stop_commands:
        Vec<Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>, STOP_COMMANDS_COUNT>,
}

impl<
        const LED_TIMER_HZ: u32,
        const RUN_COMMANDS_COUNT: usize,
        const START_COMMANDS_COUNT: usize,
        const STOP_COMMANDS_COUNT: usize,
        LedSet,
        AxisSet,
        SpindleSet,
    >
    RobotBuilder2<
        LED_TIMER_HZ,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
        LedSet,
        AxisSet,
        SpindleSet,
    >
where
    LedSet: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    AxisSet: ActuatorSet<Action = AxisAction>,
    SpindleSet: ActuatorSet<Action = SpindleAction>,
{
    pub fn new(builder1: RobotBuilder1<LedSet, AxisSet, SpindleSet>) -> Self {
        Self {
            leds: builder1.leds,
            axes: builder1.axes,
            spindles: builder1.spindles,
            run_commands: Vec::new(),
            start_commands: Vec::new(),
            stop_commands: Vec::new(),
        }
    }
}

impl<
        const LED_TIMER_HZ: u32,
        const RUN_COMMANDS_COUNT: usize,
        const START_COMMANDS_COUNT: usize,
        const STOP_COMMANDS_COUNT: usize,
        LedSet,
        AxisSet,
        SpindleSet,
    >
    RobotBuilder2<
        LED_TIMER_HZ,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
        LedSet,
        AxisSet,
        SpindleSet,
    >
where
    LedSet: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    AxisSet: ActuatorSet<Action = AxisAction>,
    SpindleSet: ActuatorSet<Action = SpindleAction>,
{
    pub fn with_run_commands(
        self,
        run_commands: &[Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>],
    ) -> Self {
        Self {
            run_commands: Vec::from_slice(run_commands).unwrap(),
            start_commands: self.start_commands,
            stop_commands: self.stop_commands,
            leds: self.leds,
            axes: self.axes,
            spindles: self.spindles,
        }
    }
    pub fn with_start_commands(
        self,
        start_commands: &[Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>],
    ) -> Self {
        Self {
            run_commands: self.run_commands,
            start_commands: Vec::from_slice(start_commands).unwrap(),
            stop_commands: self.stop_commands,
            leds: self.leds,
            axes: self.axes,
            spindles: self.spindles,
        }
    }

    pub fn with_stop_commands(
        self,
        stop_commands: &[Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>],
    ) -> Self {
        Self {
            run_commands: self.run_commands,
            start_commands: self.start_commands,
            stop_commands: Vec::from_slice(stop_commands).unwrap(),
            leds: self.leds,
            axes: self.axes,
            spindles: self.spindles,
        }
    }

    pub fn build<const ACTIVE_COMMANDS_COUNT: usize>(
        self,
    ) -> Robot<
        LED_TIMER_HZ,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
        ACTIVE_COMMANDS_COUNT,
        LedSet,
        AxisSet,
        SpindleSet,
    > {
        Robot::new(
            self.run_commands,
            self.start_commands,
            self.stop_commands,
            self.leds,
            self.axes,
            self.spindles,
        )
    }
}
