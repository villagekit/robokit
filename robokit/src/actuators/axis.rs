use core::fmt::Debug;
use core::marker::PhantomData;
use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use fugit::{TimerDurationU32 as TimerDuration, TimerInstantU32 as TimerInstant};
use fugit_timer::Timer as FugitTimer;
use stepper::{
    compat, drivers,
    motion_control::{self, SoftwareMotionControl},
    ramp_maker,
    traits::{MotionControl, SetDirection, Step},
    Direction, Stepper,
};

use super::Actuator;
use crate::sensors::{
    switch::{SwitchStatus, SwitchUpdate},
    Sensor,
};

#[derive(Clone, Copy, Debug, Format)]
pub enum AxisAction {
    MoveRelative {
        max_velocity_in_millimeters_per_sec: AxisVelocity,
        distance_in_millimeters: f64,
    },
    MoveAbsolute {
        max_velocity_in_millimeters_per_sec: AxisVelocity,
        position_in_millimeters: f64,
    },
    Home {
        max_velocity_in_millimeters_per_sec: AxisVelocity,
        back_off_distance_in_millimeters: f64,
    },
}

type AxisVelocity = f64;
type AxisMotionProfile = ramp_maker::Trapezoidal<AxisVelocity>;
type AxisMotionControl<Driver, Timer, const TIMER_HZ: u32> = SoftwareMotionControl<
    Driver,
    StepperTimer<Timer, TIMER_HZ>,
    AxisMotionProfile,
    DelayToTicks<TimerDuration<TIMER_HZ>, TIMER_HZ>,
    TIMER_HZ,
>;
type AxisDriverDQ542MA<PinDir, PinStep, Timer, const TIMER_HZ: u32> = AxisMotionControl<
    drivers::dq542ma::DQ542MA<(), compat::Pin<PinStep>, compat::Pin<PinDir>>,
    Timer,
    TIMER_HZ,
>;

// https://docs.rs/stepper/latest/src/stepper/stepper/move_to.rs.html
#[derive(Clone, Copy, Debug, Format)]
struct AxisMoveState {
    max_velocity_in_steps_per_sec: AxisVelocity,
    target_step: i32,
    #[defmt(Debug2Format)]
    direction: Direction,
}

#[derive(Clone, Copy, Debug, Format)]
enum AxisMoveStatus {
    Start,
    Motion,
}

#[derive(Clone, Copy, Debug, Format)]
struct AxisHomeState {
    max_velocity_in_steps_per_sec: AxisVelocity,
    #[defmt(Debug2Format)]
    towards_home_direction: Direction,
    #[defmt(Debug2Format)]
    back_off_home_direction: Direction,
    back_off_distance_in_steps: i32,
}

#[derive(Clone, Copy, Debug, Format)]
enum AxisHomeStatus {
    Start,
    MotionTowardsHome,
    Interlude,
    MotionBackOffHome,
    Done,
}

#[derive(Clone, Copy, Debug, Format)]
enum AxisState {
    Idle,
    Moving(AxisMoveState, AxisMoveStatus),
    Homing(AxisHomeState, AxisHomeStatus),
}

#[derive(Clone, Copy, Debug, Format, PartialEq, Eq)]
pub enum AxisLimitSide {
    Min,
    Max,
}

#[derive(Clone, Copy, Debug, Format, PartialEq, Eq)]
enum AxisLimitStatus {
    Under,
    Over,
}

pub struct AxisDevice<Driver, LimitMin, LimitMax>
where
    Driver: MotionControl,
    LimitMin: Sensor<Message = SwitchUpdate>,
    LimitMax: Sensor<Message = SwitchUpdate>,
{
    stepper: Stepper<Driver>,
    steps_per_millimeter: f64,
    state: AxisState,
    logical_position: f64,
    limit_min: LimitMin,
    limit_max: LimitMax,
    limit_min_status: Option<AxisLimitStatus>,
    limit_max_status: Option<AxisLimitStatus>,
    home_side: AxisLimitSide,
}

impl<PinDir, PinStep, Timer, const TIMER_HZ: u32, LimitMin, LimitMax>
    AxisDevice<AxisDriverDQ542MA<PinDir, PinStep, Timer, TIMER_HZ>, LimitMin, LimitMax>
where
    PinDir: OutputPin,
    <PinDir as OutputPin>::Error: Debug,
    PinStep: OutputPin,
    <PinStep as OutputPin>::Error: Debug,
    Timer: FugitTimer<TIMER_HZ>,
    <AxisDriverDQ542MA<PinDir, PinStep, Timer, TIMER_HZ> as MotionControl>::Error: Debug,
    LimitMin: Sensor<Message = SwitchUpdate>,
    LimitMax: Sensor<Message = SwitchUpdate>,
{
    pub fn new_dq542ma(
        dir: PinDir,
        step: PinStep,
        timer: Timer,
        max_acceleration_in_millimeters_per_sec_per_sec: f64,
        steps_per_millimeter: f64,
        limit_min: LimitMin,
        limit_max: LimitMax,
        home_side: AxisLimitSide,
    ) -> Self {
        let max_acceleration_in_steps_per_sec_per_sec =
            max_acceleration_in_millimeters_per_sec_per_sec * steps_per_millimeter;
        let profile = ramp_maker::Trapezoidal::new(max_acceleration_in_steps_per_sec_per_sec);

        let compat_dir = compat::Pin(dir);
        let compat_step = compat::Pin(step);
        let mut stepper_timer = StepperTimer(timer);

        let stepper = Stepper::from_driver(drivers::dq542ma::DQ542MA::new())
            .enable_direction_control(compat_dir, Direction::Forward, &mut stepper_timer)
            .unwrap()
            .enable_step_control(compat_step)
            .enable_motion_control((stepper_timer, profile, DelayToTicks::new()));

        Self {
            stepper,
            steps_per_millimeter,
            state: AxisState::Idle,
            logical_position: 0_f64,
            limit_min,
            limit_min_status: None,
            limit_max,
            limit_max_status: None,
            home_side,
        }
    }
}

impl<Driver, Timer, const TIMER_HZ: u32, LimitMin, LimitMax>
    AxisDevice<AxisMotionControl<Driver, Timer, TIMER_HZ>, LimitMin, LimitMax>
where
    Driver: SetDirection + Step,
    Timer: FugitTimer<TIMER_HZ>,
    LimitMin: Sensor<Message = SwitchUpdate>,
    LimitMax: Sensor<Message = SwitchUpdate>,
{
    pub fn get_current_step(&mut self) -> i32 {
        self.stepper.driver_mut().current_step()
    }

    pub fn get_real_position(&mut self) -> f64 {
        (self.get_current_step() as f64) / self.steps_per_millimeter
    }
}

pub struct DelayToTicks<Time, const TIMER_HZ: u32>(PhantomData<Time>);

impl<Time, const TIMER_HZ: u32> DelayToTicks<Time, TIMER_HZ> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Time, const TIMER_HZ: u32> motion_control::DelayToTicks<f64, TIMER_HZ>
    for DelayToTicks<Time, TIMER_HZ>
{
    type Error = core::convert::Infallible;

    fn delay_to_ticks(&self, delay: f64) -> Result<TimerDuration<TIMER_HZ>, Self::Error> {
        let ticks = TimerDuration::<TIMER_HZ>::from_ticks((delay * (TIMER_HZ as f64)) as u32);

        Ok(ticks)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AxisError<DriverError: Debug, LimitMinSenseError: Debug, LimitMaxSenseError: Debug> {
    DriverUpdate(DriverError),
    DriverResetPosition(DriverError),
    DriverMoveToPosition(DriverError),
    Limit(AxisLimitSide),
    LimitSensor(LimitSensorError<LimitMinSenseError, LimitMaxSenseError>),
    Unexpected,
}

impl<Driver, Timer, const TIMER_HZ: u32, LimitMin, LimitMax> Actuator
    for AxisDevice<AxisMotionControl<Driver, Timer, TIMER_HZ>, LimitMin, LimitMax>
where
    Driver: SetDirection + Step,
    Timer: FugitTimer<TIMER_HZ>,
    <AxisMotionControl<Driver, Timer, TIMER_HZ> as MotionControl>::Error: Debug,
    LimitMin: Sensor<Message = SwitchUpdate>,
    LimitMin::Error: Debug,
    LimitMax: Sensor<Message = SwitchUpdate>,
    LimitMax::Error: Debug,
{
    type Action = AxisAction;
    type Error = AxisError<
        <AxisMotionControl<Driver, Timer, TIMER_HZ> as MotionControl>::Error,
        <LimitMin as Sensor>::Error,
        <LimitMax as Sensor>::Error,
    >;

    fn run(&mut self, action: &Self::Action) {
        match action {
            AxisAction::MoveRelative {
                max_velocity_in_millimeters_per_sec,
                distance_in_millimeters,
            } => {
                let position_in_millimeters = self.logical_position + distance_in_millimeters;

                self.run(&AxisAction::MoveAbsolute {
                    max_velocity_in_millimeters_per_sec: *max_velocity_in_millimeters_per_sec,
                    position_in_millimeters,
                })
            }
            AxisAction::MoveAbsolute {
                max_velocity_in_millimeters_per_sec,
                position_in_millimeters,
            } => {
                let max_velocity_in_steps_per_sec =
                    max_velocity_in_millimeters_per_sec * self.steps_per_millimeter;

                let next_logical_position = position_in_millimeters;
                let real_position_difference = next_logical_position - self.get_real_position();
                let step_difference: i32 =
                    (real_position_difference * self.steps_per_millimeter) as i32;
                let target_step = self.get_current_step() + step_difference;

                // NOTE(mw) hmm... is this the best way to do this?
                self.logical_position = *next_logical_position;

                // NOTE(mw): We do this because stepper doesn't immediately set direction after
                //   .move_to_position(), we need the direction right away.
                let direction = if step_difference < 0 {
                    Direction::Backward
                } else {
                    Direction::Forward
                };

                self.state = AxisState::Moving(
                    AxisMoveState {
                        max_velocity_in_steps_per_sec,
                        target_step,
                        direction,
                    },
                    AxisMoveStatus::Start,
                );
            }
            AxisAction::Home {
                max_velocity_in_millimeters_per_sec,
                back_off_distance_in_millimeters,
            } => {
                let max_velocity_in_steps_per_sec =
                    max_velocity_in_millimeters_per_sec * self.steps_per_millimeter;
                let back_off_distance_in_steps =
                    (back_off_distance_in_millimeters * self.steps_per_millimeter) as i32;

                let (towards_home_direction, back_off_home_direction) = match self.home_side {
                    AxisLimitSide::Min => (Direction::Backward, Direction::Forward),
                    AxisLimitSide::Max => (Direction::Forward, Direction::Backward),
                };

                self.state = AxisState::Homing(
                    AxisHomeState {
                        max_velocity_in_steps_per_sec,
                        towards_home_direction,
                        back_off_home_direction,
                        back_off_distance_in_steps,
                    },
                    AxisHomeStatus::Start,
                );
            }
        }
    }

    // https://docs.rs/stepper/latest/src/stepper/stepper/move_to.rs.html#
    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        self.update_limit_switches()
            .map_err(AxisError::LimitSensor)?;

        // assert min and max limits are set
        if let None = self.limit_min_status {
            return Poll::Ready(Err(AxisError::Unexpected));
        }
        if let None = self.limit_max_status {
            return Poll::Ready(Err(AxisError::Unexpected));
        }

        match self.state {
            AxisState::Idle => Poll::Ready(Ok(())),
            AxisState::Moving(move_state, move_status) => {
                let AxisMoveState {
                    max_velocity_in_steps_per_sec,
                    target_step,
                    direction,
                } = move_state;

                let driver = self.stepper.driver_mut();

                match move_status {
                    AxisMoveStatus::Start => {
                        driver
                            .move_to_position(max_velocity_in_steps_per_sec, target_step)
                            .map_err(AxisError::DriverMoveToPosition)?;

                        self.state = AxisState::Moving(move_state, AxisMoveStatus::Motion);
                        Poll::Pending
                    }
                    AxisMoveStatus::Motion => {
                        match direction {
                            // limit: max
                            Direction::Forward => {
                                if let Some(AxisLimitStatus::Over) = self.limit_max_status {
                                    return Poll::Ready(Err(AxisError::Limit(AxisLimitSide::Max)));
                                }
                            }
                            // limit: min
                            Direction::Backward => {
                                if let Some(AxisLimitStatus::Over) = self.limit_min_status {
                                    return Poll::Ready(Err(AxisError::Limit(AxisLimitSide::Min)));
                                }
                            }
                        }

                        let still_moving = driver.update().map_err(AxisError::DriverUpdate)?;
                        if still_moving {
                            Poll::Pending
                        } else {
                            self.state = AxisState::Idle;
                            Poll::Ready(Ok(()))
                        }
                    }
                }
            }
            AxisState::Homing(home_state, home_status) => {
                let AxisHomeState {
                    max_velocity_in_steps_per_sec,
                    towards_home_direction,
                    back_off_home_direction,
                    back_off_distance_in_steps,
                } = home_state;

                let driver = self.stepper.driver_mut();

                match home_status {
                    AxisHomeStatus::Start => {
                        let target_step = match self.home_side {
                            AxisLimitSide::Min => i32::MIN + 1,
                            AxisLimitSide::Max => i32::MAX - 1,
                        };

                        driver
                            .reset_position(0)
                            .map_err(AxisError::DriverResetPosition)?;
                        driver
                            .move_to_position(max_velocity_in_steps_per_sec, target_step)
                            .map_err(AxisError::DriverMoveToPosition)?;

                        self.state =
                            AxisState::Homing(home_state, AxisHomeStatus::MotionTowardsHome);
                        Poll::Pending
                    }
                    AxisHomeStatus::MotionTowardsHome => {
                        let done_moving_towards_home = match towards_home_direction {
                            // limit: max
                            Direction::Forward => {
                                Some(AxisLimitStatus::Over) == self.limit_max_status
                            }
                            // limit: min
                            Direction::Backward => {
                                Some(AxisLimitStatus::Over) == self.limit_min_status
                            }
                        };

                        if done_moving_towards_home {
                            self.state = AxisState::Homing(home_state, AxisHomeStatus::Interlude);
                            return Poll::Pending;
                        }

                        let still_moving = driver.update().map_err(AxisError::DriverUpdate)?;
                        if still_moving {
                            Poll::Pending
                        } else {
                            Poll::Ready(Err(AxisError::Unexpected))
                        }
                    }
                    AxisHomeStatus::Interlude => {
                        let target_step = match self.home_side {
                            AxisLimitSide::Min => back_off_distance_in_steps,
                            AxisLimitSide::Max => -back_off_distance_in_steps,
                        };

                        driver
                            .reset_position(0)
                            .map_err(AxisError::DriverResetPosition)?;
                        driver
                            .move_to_position(max_velocity_in_steps_per_sec, target_step)
                            .map_err(AxisError::DriverMoveToPosition)?;

                        self.state =
                            AxisState::Homing(home_state, AxisHomeStatus::MotionBackOffHome);

                        Poll::Pending
                    }
                    AxisHomeStatus::MotionBackOffHome => {
                        match back_off_home_direction {
                            // limit: max
                            Direction::Forward => {
                                if let Some(AxisLimitStatus::Over) = self.limit_max_status {
                                    return Poll::Ready(Err(AxisError::Limit(AxisLimitSide::Max)));
                                }
                            }
                            // limit: min
                            Direction::Backward => {
                                if let Some(AxisLimitStatus::Over) = self.limit_min_status {
                                    return Poll::Ready(Err(AxisError::Limit(AxisLimitSide::Min)));
                                }
                            }
                        }

                        let still_moving = driver.update().map_err(AxisError::DriverUpdate)?;
                        if !still_moving {
                            self.state = AxisState::Homing(home_state, AxisHomeStatus::Done);
                        }

                        Poll::Pending
                    }
                    AxisHomeStatus::Done => {
                        driver
                            .reset_position(0)
                            .map_err(AxisError::DriverResetPosition)?;

                        self.state = AxisState::Idle;

                        self.logical_position = 0_f64;

                        Poll::Ready(Ok(()))
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LimitSensorError<LimitMinSenseError: Debug, LimitMaxSenseError: Debug> {
    Min(LimitMinSenseError),
    Max(LimitMaxSenseError),
}

impl<Driver, Timer, const TIMER_HZ: u32, LimitMin, LimitMax>
    AxisDevice<AxisMotionControl<Driver, Timer, TIMER_HZ>, LimitMin, LimitMax>
where
    Driver: SetDirection + Step,
    Timer: FugitTimer<TIMER_HZ>,
    LimitMin: Sensor<Message = SwitchUpdate>,
    LimitMin::Error: Debug,
    LimitMax: Sensor<Message = SwitchUpdate>,
    LimitMax::Error: Debug,
{
    pub fn update_limit_switches(
        &mut self,
    ) -> Result<(), LimitSensorError<LimitMin::Error, LimitMax::Error>> {
        if let Some(axis_limit_update) = self.limit_min.sense().map_err(LimitSensorError::Min)? {
            let limit_min_status = match axis_limit_update.status {
                SwitchStatus::On => AxisLimitStatus::Over,
                SwitchStatus::Off => AxisLimitStatus::Under,
            };
            self.limit_min_status = Some(limit_min_status);
        }

        if let Some(axis_limit_update) = self.limit_max.sense().map_err(LimitSensorError::Max)? {
            let limit_max_status = match axis_limit_update.status {
                SwitchStatus::On => AxisLimitStatus::Over,
                SwitchStatus::Off => AxisLimitStatus::Under,
            };
            self.limit_max_status = Some(limit_max_status);
        }

        Ok(())
    }
}

pub struct StepperTimer<Timer, const TIMER_HZ: u32>(pub Timer);

impl<Timer, const TIMER_HZ: u32> FugitTimer<TIMER_HZ> for StepperTimer<Timer, TIMER_HZ>
where
    Timer: FugitTimer<TIMER_HZ>,
{
    type Error = Timer::Error;

    fn now(&mut self) -> TimerInstant<TIMER_HZ> {
        self.0.now()
    }

    fn start(&mut self, mut duration: TimerDuration<TIMER_HZ>) -> Result<(), Self::Error> {
        // wait to discard any interrupt events that triggered before we started.
        self.0.wait().ok();

        // if below minimum, set to minimum: 2 ticks
        let minimum_duration = TimerDuration::<TIMER_HZ>::from_ticks(2);
        if duration < minimum_duration {
            duration = minimum_duration;
        }

        self.0.start(duration)
    }

    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.0.cancel()
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        self.0.wait()
    }
}
