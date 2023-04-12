use alloc::boxed::Box;
use alloc::string::String;
use core::task::Poll;
use heapless::{FnvIndexMap, Vec};

use crate::actuators::BoxifyActuator;
use crate::actuators::{
    axis::AxisAction, led::LedAction, spindle::SpindleAction, Actuator, BoxActuator,
};
use crate::error::Error;
use crate::runner::{Command, Runner};
use crate::scheduler::Scheduler;

pub struct RobotBuilder<
    'a,
    const LED_TIMER_HZ: u32,
    const RUN_COMMANDS_COUNT: usize,
    const START_COMMANDS_COUNT: usize,
    const STOP_COMMANDS_COUNT: usize,
    const LEDS_COUNT: usize,
    const AXES_COUNT: usize,
    const SPINDLES_COUNT: usize,
    const ACTIVE_COMMANDS_COUNT: usize,
> {
    run_commands: Vec<Command<'a, LED_TIMER_HZ>, RUN_COMMANDS_COUNT>,
    start_commands: Vec<Command<'a, LED_TIMER_HZ>, START_COMMANDS_COUNT>,
    stop_commands: Vec<Command<'a, LED_TIMER_HZ>, STOP_COMMANDS_COUNT>,
    leds: FnvIndexMap<&'a str, BoxActuator<LedAction<LED_TIMER_HZ>>, LEDS_COUNT>,
    axes: FnvIndexMap<&'a str, BoxActuator<AxisAction>, AXES_COUNT>,
    spindles: FnvIndexMap<&'a str, BoxActuator<SpindleAction>, SPINDLES_COUNT>,
}

pub struct Robot<
    'a,
    const LED_TIMER_HZ: u32,
    const RUN_COMMANDS_COUNT: usize,
    const START_COMMANDS_COUNT: usize,
    const STOP_COMMANDS_COUNT: usize,
    const LEDS_COUNT: usize,
    const AXES_COUNT: usize,
    const SPINDLES_COUNT: usize,
    const ACTIVE_COMMANDS_COUNT: usize,
> {
    scheduler: Scheduler<
        Command<'a, LED_TIMER_HZ>,
        Runner<'a, LED_TIMER_HZ, LEDS_COUNT, AXES_COUNT, SPINDLES_COUNT, ACTIVE_COMMANDS_COUNT>,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
    >,
}

#[derive(Copy, Clone, Debug)]
pub enum RobotBuilderError {
    TooManyLeds,
    TooManyAxes,
    TooManySpindles,
    TooManyRunCommands,
    TooManyStartCommands,
    TooManyStopCommands,
}

#[derive(Clone, Debug)]
pub enum RobotValidationError {
    UnmatchedId {
        id: String,
        actuator_type: String,
        command_type: String,
    },
}

impl<
        'a,
        const LED_TIMER_HZ: u32,
        const RUN_COMMANDS_COUNT: usize,
        const START_COMMANDS_COUNT: usize,
        const STOP_COMMANDS_COUNT: usize,
        const LEDS_COUNT: usize,
        const AXES_COUNT: usize,
        const SPINDLES_COUNT: usize,
        const ACTIVE_COMMANDS_COUNT: usize,
    >
    RobotBuilder<
        'a,
        LED_TIMER_HZ,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
        LEDS_COUNT,
        AXES_COUNT,
        SPINDLES_COUNT,
        ACTIVE_COMMANDS_COUNT,
    >
{
    pub fn new() -> Self {
        Self {
            run_commands: Vec::new(),
            start_commands: Vec::new(),
            stop_commands: Vec::new(),
            leds: FnvIndexMap::new(),
            axes: FnvIndexMap::new(),
            spindles: FnvIndexMap::new(),
        }
    }

    pub fn add_led<A: Actuator<Action = LedAction<LED_TIMER_HZ>> + 'static>(
        &mut self,
        id: &'a str,
        actuator: A,
    ) -> Result<(), RobotBuilderError> {
        self.leds
            .insert(id, Box::new(BoxifyActuator::new(actuator)))
            .map_err(|_| RobotBuilderError::TooManyLeds)?;

        Ok(())
    }

    pub fn add_axis<A: Actuator<Action = AxisAction> + 'static>(
        &mut self,
        id: &'a str,
        actuator: A,
    ) -> Result<(), RobotBuilderError> {
        self.axes
            .insert(id, Box::new(BoxifyActuator::new(actuator)))
            .map_err(|_| RobotBuilderError::TooManyAxes)?;

        Ok(())
    }

    pub fn add_spindle<A: Actuator<Action = SpindleAction> + 'static>(
        &mut self,
        id: &'a str,
        actuator: A,
    ) -> Result<(), RobotBuilderError> {
        self.spindles
            .insert(id, Box::new(BoxifyActuator::new(actuator)))
            .map_err(|_| RobotBuilderError::TooManySpindles)?;

        Ok(())
    }

    pub fn set_run_commands(
        &mut self,
        run_commands: &[Command<'a, LED_TIMER_HZ>],
    ) -> Result<(), RobotBuilderError> {
        self.run_commands =
            Vec::from_slice(run_commands).map_err(|_| RobotBuilderError::TooManyRunCommands)?;

        Ok(())
    }

    pub fn set_start_commands(
        &mut self,
        start_commands: &[Command<'a, LED_TIMER_HZ>],
    ) -> Result<(), RobotBuilderError> {
        self.start_commands =
            Vec::from_slice(start_commands).map_err(|_| RobotBuilderError::TooManyStartCommands)?;

        Ok(())
    }

    pub fn set_stop_commands(
        &mut self,
        stop_commands: &[Command<'a, LED_TIMER_HZ>],
    ) -> Result<(), RobotBuilderError> {
        self.stop_commands =
            Vec::from_slice(stop_commands).map_err(|_| RobotBuilderError::TooManyStopCommands)?;

        Ok(())
    }

    fn validate_commands<const COMMANDS_COUNT: usize>(
        &self,
        command_type: &'a str,
        commands: &Vec<Command<'a, LED_TIMER_HZ>, COMMANDS_COUNT>,
    ) -> Result<(), RobotValidationError> {
        for command in commands.iter() {
            match command {
                Command::Led(id, _) => {
                    if !self.leds.contains_key(*id) {
                        return Err(RobotValidationError::UnmatchedId {
                            id: (*id).into(),
                            actuator_type: "led".into(),
                            command_type: command_type.into(),
                        });
                    }
                }
                Command::Axis(id, _) => {
                    if !self.axes.contains_key(*id) {
                        return Err(RobotValidationError::UnmatchedId {
                            id: (*id).into(),
                            actuator_type: "axis".into(),
                            command_type: command_type.into(),
                        });
                    }
                }
                Command::Spindle(id, _) => {
                    if !self.spindles.contains_key(*id) {
                        return Err(RobotValidationError::UnmatchedId {
                            id: (*id).into(),
                            actuator_type: "spindle".into(),
                            command_type: command_type.into(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn validate(&self) -> Result<(), RobotValidationError> {
        self.validate_commands("run", &self.run_commands)?;
        self.validate_commands("start", &self.start_commands)?;
        self.validate_commands("stop", &self.stop_commands)?;

        Ok(())
    }

    pub fn build(
        self,
    ) -> Result<
        Robot<
            'a,
            LED_TIMER_HZ,
            RUN_COMMANDS_COUNT,
            START_COMMANDS_COUNT,
            STOP_COMMANDS_COUNT,
            LEDS_COUNT,
            AXES_COUNT,
            SPINDLES_COUNT,
            ACTIVE_COMMANDS_COUNT,
        >,
        RobotValidationError,
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
        'a,
        const LED_TIMER_HZ: u32,
        const RUN_COMMANDS_COUNT: usize,
        const START_COMMANDS_COUNT: usize,
        const STOP_COMMANDS_COUNT: usize,
        const LEDS_COUNT: usize,
        const AXES_COUNT: usize,
        const SPINDLES_COUNT: usize,
        const ACTIVE_COMMANDS_COUNT: usize,
    >
    Robot<
        'a,
        LED_TIMER_HZ,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
        LEDS_COUNT,
        AXES_COUNT,
        SPINDLES_COUNT,
        ACTIVE_COMMANDS_COUNT,
    >
{
    pub fn new(
        scheduler: Scheduler<
            Command<'a, LED_TIMER_HZ>,
            Runner<'a, LED_TIMER_HZ, LEDS_COUNT, AXES_COUNT, SPINDLES_COUNT, ACTIVE_COMMANDS_COUNT>,
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
