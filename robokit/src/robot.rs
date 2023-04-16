use alloc::boxed::Box;
use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use fixed_map::{key::Key, Map};
use heapless::Vec;

use crate::actuators::BoxifyActuator;
use crate::actuators::{
    axis::AxisAction, led::LedAction, spindle::SpindleAction, Actuator, BoxActuator,
};
use crate::error::Error;
use crate::runner::{Command, Runner};
use crate::scheduler::Scheduler;

pub struct RobotBuilder<
    const LED_TIMER_HZ: u32,
    const RUN_COMMANDS_COUNT: usize,
    const START_COMMANDS_COUNT: usize,
    const STOP_COMMANDS_COUNT: usize,
    const ACTIVE_COMMANDS_COUNT: usize,
    LedId,
    AxisId,
    SpindleId,
> where
    LedId: Key + Debug + Format,
    AxisId: Key + Debug + Format,
    SpindleId: Key + Debug + Format,
{
    run_commands: Vec<Command<LED_TIMER_HZ, LedId, AxisId, SpindleId>, RUN_COMMANDS_COUNT>,
    start_commands: Vec<Command<LED_TIMER_HZ, LedId, AxisId, SpindleId>, START_COMMANDS_COUNT>,
    stop_commands: Vec<Command<LED_TIMER_HZ, LedId, AxisId, SpindleId>, STOP_COMMANDS_COUNT>,
    leds: Map<LedId, BoxActuator<LedAction<LED_TIMER_HZ>>>,
    axes: Map<AxisId, BoxActuator<AxisAction>>,
    spindles: Map<SpindleId, BoxActuator<SpindleAction>>,
}

pub struct Robot<
    const LED_TIMER_HZ: u32,
    const RUN_COMMANDS_COUNT: usize,
    const START_COMMANDS_COUNT: usize,
    const STOP_COMMANDS_COUNT: usize,
    const ACTIVE_COMMANDS_COUNT: usize,
    LedId,
    AxisId,
    SpindleId,
> where
    LedId: Key + Debug + Format,
    AxisId: Key + Debug + Format,
    SpindleId: Key + Debug + Format,
{
    scheduler: Scheduler<
        Command<LED_TIMER_HZ, LedId, AxisId, SpindleId>,
        Runner<LED_TIMER_HZ, ACTIVE_COMMANDS_COUNT, LedId, AxisId, SpindleId>,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
    >,
}

#[derive(Copy, Clone, Debug)]
pub enum RobotBuilderError {
    TooManyRunCommands,
    TooManyStartCommands,
    TooManyStopCommands,
}

#[derive(Clone, Debug)]
pub enum RobotValidationError<LedId: Debug, AxisId: Debug, SpindleId: Debug> {
    UnmatchedLedId {
        id: LedId,
        command_type: &'static str,
    },
    UnmatchedAxisId {
        id: AxisId,
        command_type: &'static str,
    },
    UnmatchedSpindleId {
        id: SpindleId,
        command_type: &'static str,
    },
}

impl<
        const LED_TIMER_HZ: u32,
        const RUN_COMMANDS_COUNT: usize,
        const START_COMMANDS_COUNT: usize,
        const STOP_COMMANDS_COUNT: usize,
        const ACTIVE_COMMANDS_COUNT: usize,
        LedId,
        AxisId,
        SpindleId,
    >
    RobotBuilder<
        LED_TIMER_HZ,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
        ACTIVE_COMMANDS_COUNT,
        LedId,
        AxisId,
        SpindleId,
    >
where
    LedId: Key + Debug + Format,
    AxisId: Key + Debug + Format,
    SpindleId: Key + Debug + Format,
{
    pub fn new() -> Self {
        Self {
            run_commands: Vec::new(),
            start_commands: Vec::new(),
            stop_commands: Vec::new(),
            leds: Map::new(),
            axes: Map::new(),
            spindles: Map::new(),
        }
    }

    pub fn add_led<A: Actuator<Action = LedAction<LED_TIMER_HZ>> + 'static>(
        &mut self,
        id: LedId,
        actuator: A,
    ) -> Result<(), RobotBuilderError> {
        self.leds
            .insert(id, Box::new(BoxifyActuator::new(actuator)));

        Ok(())
    }

    pub fn add_axis<A: Actuator<Action = AxisAction> + 'static>(
        &mut self,
        id: AxisId,
        actuator: A,
    ) -> Result<(), RobotBuilderError> {
        self.axes
            .insert(id, Box::new(BoxifyActuator::new(actuator)));

        Ok(())
    }

    pub fn add_spindle<A: Actuator<Action = SpindleAction> + 'static>(
        &mut self,
        id: SpindleId,
        actuator: A,
    ) -> Result<(), RobotBuilderError> {
        self.spindles
            .insert(id, Box::new(BoxifyActuator::new(actuator)));

        Ok(())
    }

    pub fn set_run_commands(
        &mut self,
        run_commands: &[Command<LED_TIMER_HZ, LedId, AxisId, SpindleId>],
    ) -> Result<(), RobotBuilderError> {
        self.run_commands =
            Vec::from_slice(run_commands).map_err(|_| RobotBuilderError::TooManyRunCommands)?;

        Ok(())
    }

    pub fn set_start_commands(
        &mut self,
        start_commands: &[Command<LED_TIMER_HZ, LedId, AxisId, SpindleId>],
    ) -> Result<(), RobotBuilderError> {
        self.start_commands =
            Vec::from_slice(start_commands).map_err(|_| RobotBuilderError::TooManyStartCommands)?;

        Ok(())
    }

    pub fn set_stop_commands(
        &mut self,
        stop_commands: &[Command<LED_TIMER_HZ, LedId, AxisId, SpindleId>],
    ) -> Result<(), RobotBuilderError> {
        self.stop_commands =
            Vec::from_slice(stop_commands).map_err(|_| RobotBuilderError::TooManyStopCommands)?;

        Ok(())
    }

    fn validate_commands<const COMMANDS_COUNT: usize>(
        &self,
        command_type: &'static str,
        commands: &Vec<Command<LED_TIMER_HZ, LedId, AxisId, SpindleId>, COMMANDS_COUNT>,
    ) -> Result<(), RobotValidationError<LedId, AxisId, SpindleId>> {
        for command in commands.iter() {
            match *command {
                Command::Led(id, _) => {
                    if !self.leds.contains_key(id) {
                        return Err(RobotValidationError::UnmatchedLedId { id, command_type });
                    }
                }
                Command::Axis(id, _) => {
                    if !self.axes.contains_key(id) {
                        return Err(RobotValidationError::UnmatchedAxisId { id, command_type });
                    }
                }
                Command::Spindle(id, _) => {
                    if !self.spindles.contains_key(id) {
                        return Err(RobotValidationError::UnmatchedSpindleId { id, command_type });
                    }
                }
            }
        }

        Ok(())
    }

    fn validate(&self) -> Result<(), RobotValidationError<LedId, AxisId, SpindleId>> {
        self.validate_commands("run", &self.run_commands)?;
        self.validate_commands("start", &self.start_commands)?;
        self.validate_commands("stop", &self.stop_commands)?;

        Ok(())
    }
    pub fn build(
        self,
    ) -> Result<
        Robot<
            LED_TIMER_HZ,
            RUN_COMMANDS_COUNT,
            START_COMMANDS_COUNT,
            STOP_COMMANDS_COUNT,
            ACTIVE_COMMANDS_COUNT,
            LedId,
            AxisId,
            SpindleId,
        >,
        RobotValidationError<LedId, AxisId, SpindleId>,
    > {
        self.validate()?;

        let runner = Runner::new(self.leds, self.axes, self.spindles);
        let scheduler = Scheduler::new(
            runner,
            self.run_commands,
            self.start_commands,
            self.stop_commands,
        );

        Ok(Robot::new(scheduler))
    }
}

impl<
        const LED_TIMER_HZ: u32,
        const RUN_COMMANDS_COUNT: usize,
        const START_COMMANDS_COUNT: usize,
        const STOP_COMMANDS_COUNT: usize,
        const ACTIVE_COMMANDS_COUNT: usize,
        LedId,
        AxisId,
        SpindleId,
    >
    Robot<
        LED_TIMER_HZ,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
        ACTIVE_COMMANDS_COUNT,
        LedId,
        AxisId,
        SpindleId,
    >
where
    LedId: Key + Debug + Format,
    AxisId: Key + Debug + Format,
    SpindleId: Key + Debug + Format,
{
    pub fn new(
        scheduler: Scheduler<
            Command<LED_TIMER_HZ, LedId, AxisId, SpindleId>,
            Runner<LED_TIMER_HZ, ACTIVE_COMMANDS_COUNT, LedId, AxisId, SpindleId>,
            RUN_COMMANDS_COUNT,
            START_COMMANDS_COUNT,
            STOP_COMMANDS_COUNT,
        >,
    ) -> Self {
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
