use alloc::boxed::Box;
use core::task::Poll;
use heapless::{FnvIndexMap, Vec};

use crate::actuators::BoxifyActuator;
use crate::actuators::{
    axis::AxisAction, led::LedAction, spindle::SpindleAction, Actuator, BoxActuator,
};
use crate::error::Error;
use crate::runner::{Command, Runner};
use crate::scheduler::Scheduler;

pub struct RobotBuilder<'a, const LED_TIMER_HZ: u32> {
    run_commands: Vec<Command<'a, LED_TIMER_HZ>, 32>,
    start_commands: Vec<Command<'a, LED_TIMER_HZ>, 4>,
    stop_commands: Vec<Command<'a, LED_TIMER_HZ>, 4>,
    leds: FnvIndexMap<&'a str, BoxActuator<LedAction<LED_TIMER_HZ>>, 16>,
    axes: FnvIndexMap<&'a str, BoxActuator<AxisAction>, 16>,
    spindles: FnvIndexMap<&'a str, BoxActuator<SpindleAction>, 16>,
}

pub struct Robot<'a, const LED_TIMER_HZ: u32> {
    scheduler: Scheduler<Command<'a, LED_TIMER_HZ>, Runner<'a, LED_TIMER_HZ>>,
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

impl<'a, const LED_TIMER_HZ: u32> RobotBuilder<'a, LED_TIMER_HZ> {
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

    pub fn build(self) -> Robot<'a, LED_TIMER_HZ> {
        let runner = Runner::new(self.leds, self.axes, self.spindles);
        let scheduler = Scheduler::new(
            runner,
            self.run_commands,
            self.start_commands,
            self.stop_commands,
        );
        Robot::new(scheduler)
    }
}

impl<'a, const LED_TIMER_HZ: u32> Robot<'a, LED_TIMER_HZ> {
    pub fn new(scheduler: Scheduler<Command<'a, LED_TIMER_HZ>, Runner<'a, LED_TIMER_HZ>>) -> Self {
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
