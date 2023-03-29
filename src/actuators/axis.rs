use anyhow::{anyhow, Error};
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

use crate::actor::{ActorPoll, ActorReceive};

pub type AxisMotionProfile = ramp_maker::Trapezoidal<f64>;
pub type AxisMotionControl<Driver, Timer, const TIMER_HZ: u32> = SoftwareMotionControl<
    Driver,
    StepperTimer<Timer, TIMER_HZ>,
    AxisMotionProfile,
    DelayToTicks<TimerDuration<TIMER_HZ>, TIMER_HZ>,
    TIMER_HZ,
>;

pub type AxisDriverDQ542MA<PinDir, PinStep, Timer, const TIMER_HZ: u32> = AxisMotionControl<
    drivers::dq542ma::DQ542MA<(), compat::Pin<PinStep>, compat::Pin<PinDir>>,
    Timer,
    TIMER_HZ,
>;
pub type AxisDriverErrorDQ542MA<PinDir, PinStep, Timer, const TIMER_HZ: u32> =
    <AxisDriverDQ542MA<PinDir, PinStep, Timer, TIMER_HZ> as MotionControl>::Error;

// https://docs.rs/stepper/latest/src/stepper/stepper/move_to.rs.html
#[derive(Clone, Copy, Debug, Format)]
pub enum AxisState<Velocity> {
    Idle,
    Initial {
        max_velocity_in_steps_per_sec: Velocity,
        target_step: i32,
    },
    Moving,
}

#[derive(Clone, Copy, Debug, Format)]
pub enum AxisLimitSide {
    Min,
    Max,
}

#[derive(Clone, Copy, Debug, Format)]
pub enum AxisLimitStatus {
    Under,
    Over,
}

pub struct Axis<Driver>
where
    Driver: MotionControl,
{
    stepper: Stepper<Driver>,
    steps_per_millimeter: f64,
    state: AxisState<<Driver as MotionControl>::Velocity>,
    logical_position: f64,
    limit_min: Option<AxisLimitStatus>,
    limit_max: Option<AxisLimitStatus>,
}

impl<PinDir, PinStep, Timer, const TIMER_HZ: u32>
    Axis<AxisDriverDQ542MA<PinDir, PinStep, Timer, TIMER_HZ>>
where
    PinDir: OutputPin,
    <PinDir as OutputPin>::Error: Debug,
    PinStep: OutputPin,
    <PinStep as OutputPin>::Error: Debug,
    Timer: FugitTimer<TIMER_HZ>,
    <AxisDriverDQ542MA<PinDir, PinStep, Timer, TIMER_HZ> as MotionControl>::Error: Debug,
{
    pub fn new_dq542ma(
        dir: PinDir,
        step: PinStep,
        timer: Timer,
        max_acceleration_in_millimeters_per_sec_per_sec: f64,
        steps_per_millimeter: f64,
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

        Axis {
            stepper: stepper,
            steps_per_millimeter,
            state: AxisState::Idle,
            logical_position: 0_f64,
            limit_min: None,
            limit_max: None,
        }
    }
}

impl<Driver, Timer, const TIMER_HZ: u32> Axis<AxisMotionControl<Driver, Timer, TIMER_HZ>>
where
    Driver: SetDirection + Step,
    Timer: FugitTimer<TIMER_HZ>,
{
    pub fn get_real_position(&mut self) -> f64 {
        (self.stepper.driver_mut().current_step() as f64) / self.steps_per_millimeter
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

#[derive(Clone, Copy, Debug, Format)]
pub struct AxisMoveMessage {
    pub max_velocity_in_millimeters_per_sec: f64,
    pub distance_in_millimeters: f64,
}

impl<Driver, Timer, const TIMER_HZ: u32> ActorReceive<AxisMoveMessage>
    for Axis<AxisMotionControl<Driver, Timer, TIMER_HZ>>
where
    Driver: SetDirection + Step,
    Timer: FugitTimer<TIMER_HZ>,
{
    fn receive(&mut self, action: &AxisMoveMessage) {
        let max_velocity_in_steps_per_sec =
            action.max_velocity_in_millimeters_per_sec * self.steps_per_millimeter;

        let distance_in_millimeters = action.distance_in_millimeters;

        let next_logical_position = self.logical_position + distance_in_millimeters;
        let real_position_difference = next_logical_position - self.get_real_position();
        let step_difference: i32 = (real_position_difference * self.steps_per_millimeter) as i32;
        let target_step = self.stepper.driver_mut().current_step() + step_difference;

        // NOTE(mw) hmm... is this the best way to do this?
        self.logical_position = next_logical_position;

        self.state = AxisState::Initial {
            max_velocity_in_steps_per_sec,
            target_step,
        };
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct AxisLimitMessage {
    pub side: AxisLimitSide,
    pub status: AxisLimitStatus,
}

impl<Driver> ActorReceive<AxisLimitMessage> for Axis<Driver>
where
    Driver: MotionControl,
{
    fn receive(&mut self, action: &AxisLimitMessage) {
        match action.side {
            AxisLimitSide::Min => {
                self.limit_min = Some(action.status);
            }
            AxisLimitSide::Max => {
                self.limit_max = Some(action.status);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AxisError<DriverError: Debug> {
    Driver(DriverError),
    Limit(AxisLimitSide),
    Unexpected,
}

// https://docs.rs/stepper/latest/src/stepper/stepper/move_to.rs.html#
impl<Driver, Timer, const TIMER_HZ: u32> ActorPoll
    for Axis<AxisMotionControl<Driver, Timer, TIMER_HZ>>
where
    Driver: SetDirection + Step,
    Timer: FugitTimer<TIMER_HZ>,
    <AxisMotionControl<Driver, Timer, TIMER_HZ> as MotionControl>::Error: Debug,
{
    fn poll(&mut self) -> Poll<Result<(), Error>> {
        // limit: min
        if let None = self.limit_min {
            return Poll::Ready(Err(AxisError::Unexpected));
        }
        if let Some(AxisLimitStatus::Over) = self.limit_min {
            if let Direction::Backward = self.stepper.driver_mut().current_direction() {
                return Poll::Ready(Err(anyhow!(AxisError::Limit(AxisLimitSide::Min))));
            }
        }

        // limit: max
        if let None = self.limit_max {
            return Poll::Ready(Err(AxisError::Unexpected));
        }
        if let Some(AxisLimitStatus::Over) = self.limit_max {
            if let Direction::Forward = self.stepper.driver_mut().current_direction() {
                return Poll::Ready(Err(anyhow!(AxisError::Limit(AxisLimitSide::Max))));
            }
        }

        match self.state {
            AxisState::Idle => Poll::Ready(Ok(())),
            AxisState::Initial {
                max_velocity_in_steps_per_sec,
                target_step,
            } => {
                self.stepper
                    .driver_mut()
                    .move_to_position(max_velocity_in_steps_per_sec, target_step)
                    .map_err(|err| AxisError::Driver(err))
                    .map_err(Error::msg)?;
                self.state = AxisState::Moving;
                Poll::Pending
            }
            AxisState::Moving => {
                let still_moving = self
                    .stepper
                    .driver_mut()
                    .update()
                    .map_err(|err| AxisError::Driver(err))
                    .map_err(Error::msg)?;
                if still_moving {
                    Poll::Pending
                } else {
                    self.state = AxisState::Idle;
                    Poll::Ready(Ok(()))
                }
            }
        }
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
