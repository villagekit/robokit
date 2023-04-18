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

    pub fn builder() -> RobotBuilder<(), (), (), (), (), ()> {
        RobotBuilder::new()
    }
}

pub struct RobotBuilder<RunCommands, StartCommands, StopCommands, LedSet, AxisSet, SpindleSet> {
    run_commands: RunCommands,
    start_commands: StartCommands,
    stop_commands: StopCommands,
    leds: LedSet,
    axes: AxisSet,
    spindles: SpindleSet,
}

impl RobotBuilder<(), (), (), (), (), ()> {
    pub fn new() -> Self {
        Self {
            run_commands: (),
            start_commands: (),
            stop_commands: (),
            leds: (),
            axes: (),
            spindles: (),
        }
    }
}

impl<const LED_TIMER_HZ: u32, LedSet, AxisSet, SpindleSet>
    RobotBuilder<
        &[Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>],
        &[Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>],
        &[Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>],
        LedSet,
        AxisSet,
        SpindleSet,
    >
where
    LedSet: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    AxisSet: ActuatorSet<Action = AxisAction>,
    SpindleSet: ActuatorSet<Action = SpindleAction>,
{
    pub fn build<
        const RUN_COMMANDS_COUNT: usize,
        const START_COMMANDS_COUNT: usize,
        const STOP_COMMANDS_COUNT: usize,
        const ACTIVE_COMMANDS_COUNT: usize,
    >(
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
            Vec::from_slice(self.run_commands).unwrap(),
            Vec::from_slice(self.start_commands).unwrap(),
            Vec::from_slice(self.stop_commands).unwrap(),
            self.leds,
            self.axes,
            self.spindles,
        )
    }
}

impl<StartCommands, StopCommands, LedSet, AxisSet, SpindleSet>
    RobotBuilder<(), StartCommands, StopCommands, LedSet, AxisSet, SpindleSet>
{
    pub fn with_run_commands<RunCommands>(
        self,
        run_commands: RunCommands,
    ) -> RobotBuilder<RunCommands, StartCommands, StopCommands, LedSet, AxisSet, SpindleSet> {
        RobotBuilder {
            run_commands,
            start_commands: self.start_commands,
            stop_commands: self.stop_commands,
            leds: self.leds,
            axes: self.axes,
            spindles: self.spindles,
        }
    }
}

impl<RunCommands, StopCommands, LedSet, AxisSet, SpindleSet>
    RobotBuilder<RunCommands, (), StopCommands, LedSet, AxisSet, SpindleSet>
{
    pub fn with_start_commands<StartCommands>(
        self,
        start_commands: StartCommands,
    ) -> RobotBuilder<RunCommands, StartCommands, StopCommands, LedSet, AxisSet, SpindleSet> {
        RobotBuilder {
            run_commands: self.run_commands,
            start_commands,
            stop_commands: self.stop_commands,
            leds: self.leds,
            axes: self.axes,
            spindles: self.spindles,
        }
    }
}

impl<RunCommands, StartCommands, LedSet, AxisSet, SpindleSet>
    RobotBuilder<RunCommands, StartCommands, (), LedSet, AxisSet, SpindleSet>
{
    pub fn with_stop_commands<StopCommands>(
        self,
        stop_commands: StopCommands,
    ) -> RobotBuilder<RunCommands, StartCommands, StopCommands, LedSet, AxisSet, SpindleSet> {
        RobotBuilder {
            run_commands: self.run_commands,
            start_commands: self.start_commands,
            stop_commands,
            leds: self.leds,
            axes: self.axes,
            spindles: self.spindles,
        }
    }
}

impl<RunCommands, StartCommands, StopCommands, AxisSet, SpindleSet>
    RobotBuilder<RunCommands, StartCommands, StopCommands, (), AxisSet, SpindleSet>
{
    pub fn with_leds<const LED_TIMER_HZ: u32, LedSet>(
        self,
        leds: LedSet,
    ) -> RobotBuilder<RunCommands, StartCommands, StopCommands, LedSet, AxisSet, SpindleSet>
    where
        LedSet: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    {
        RobotBuilder {
            run_commands: self.run_commands,
            start_commands: self.start_commands,
            stop_commands: self.stop_commands,
            leds,
            axes: self.axes,
            spindles: self.spindles,
        }
    }
}

impl<RunCommands, StartCommands, StopCommands, LedSet, SpindleSet>
    RobotBuilder<RunCommands, StartCommands, StopCommands, LedSet, (), SpindleSet>
{
    pub fn with_axes<AxisSet>(
        self,
        axes: AxisSet,
    ) -> RobotBuilder<RunCommands, StartCommands, StopCommands, LedSet, AxisSet, SpindleSet>
    where
        AxisSet: ActuatorSet<Action = AxisAction>,
    {
        RobotBuilder {
            run_commands: self.run_commands,
            start_commands: self.start_commands,
            stop_commands: self.stop_commands,
            leds: self.leds,
            axes,
            spindles: self.spindles,
        }
    }
}

impl<RunCommands, StartCommands, StopCommands, LedSet, AxisSet>
    RobotBuilder<RunCommands, StartCommands, StopCommands, LedSet, AxisSet, ()>
{
    pub fn with_spindles<SpindleSet>(
        self,
        spindles: SpindleSet,
    ) -> RobotBuilder<RunCommands, StartCommands, StopCommands, LedSet, AxisSet, SpindleSet>
    where
        SpindleSet: ActuatorSet<Action = SpindleAction>,
    {
        RobotBuilder {
            run_commands: self.run_commands,
            start_commands: self.start_commands,
            stop_commands: self.stop_commands,
            leds: self.leds,
            axes: self.axes,
            spindles,
        }
    }
}
