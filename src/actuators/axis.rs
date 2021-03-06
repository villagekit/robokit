use core::convert::Infallible;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops;
use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use fugit::{TimerDuration, TimerDurationU32};
use fugit_timer::Timer as FugitTimer;
use nb;
use stepper::embedded_hal::timer::nb::CountDown;
use stepper::embedded_time::{duration::*, ConversionError};
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
    DelayToTicks<TimerDurationU32<TIMER_HZ>, TIMER_HZ>,
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
        let mut compat_timer = StepperTimer(timer);

        let stepper = Stepper::from_driver(drivers::dq542ma::DQ542MA::new())
            .enable_direction_control(compat_dir, Direction::Forward, &mut compat_timer)
            .unwrap()
            .enable_step_control(compat_step)
            .enable_motion_control((compat_timer, profile, DelayToTicks::new()));

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

impl<Time, const TIMER_HZ: u32> motion_control::DelayToTicks<f64> for DelayToTicks<Time, TIMER_HZ> {
    type Ticks = StepperTicks<TIMER_HZ>;
    type Error = core::convert::Infallible;

    fn delay_to_ticks(&self, delay: f64) -> Result<Self::Ticks, Self::Error> {
        let ticks = TimerDurationU32::<TIMER_HZ>::from_ticks((delay * (TIMER_HZ as f64)) as u32);

        Ok(StepperTicks::<TIMER_HZ>(ticks))
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
    type Error = AxisError<<AxisMotionControl<Driver, Timer, TIMER_HZ> as MotionControl>::Error>;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        // limit: min
        if let None = self.limit_min {
            return Poll::Ready(Err(AxisError::Unexpected));
        }
        if let Some(AxisLimitStatus::Over) = self.limit_min {
            if let Direction::Backward = self.stepper.driver_mut().current_direction() {
                return Poll::Ready(Err(AxisError::Limit(AxisLimitSide::Min)));
            }
        }

        // limit: max
        if let None = self.limit_max {
            return Poll::Ready(Err(AxisError::Unexpected));
        }
        if let Some(AxisLimitStatus::Over) = self.limit_max {
            if let Direction::Forward = self.stepper.driver_mut().current_direction() {
                return Poll::Ready(Err(AxisError::Limit(AxisLimitSide::Max)));
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
                    .map_err(|err| AxisError::Driver(err))?;
                self.state = AxisState::Moving;
                Poll::Pending
            }
            AxisState::Moving => {
                let still_moving = self
                    .stepper
                    .driver_mut()
                    .update()
                    .map_err(|err| AxisError::Driver(err))?;
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

impl<Timer, const TIMER_HZ: u32> CountDown for StepperTimer<Timer, TIMER_HZ>
where
    Timer: FugitTimer<TIMER_HZ>,
{
    type Error = Infallible;

    type Time = StepperTicks<TIMER_HZ>;

    fn start<Ticks>(&mut self, ticks: Ticks) -> Result<(), Self::Error>
    where
        Ticks: Into<Self::Time>,
    {
        /*
        // subtract time spent between now and previous timer
        let sofar_ticks = self.0.now().ticks();
        let wait_ticks = ticks.into().0.ticks();
        let mut ticks = if wait_ticks <= sofar_ticks {
            0
        } else {
            wait_ticks - sofar_ticks
        };
        if ticks > 50 {
            defmt::println!("timer: {} - {} = {}", wait_ticks, sofar_ticks, ticks);
        }
        */

        let mut ticks = ticks.into().0.ticks();

        // wait to discard any interrupt events that triggered before we started.
        self.0.wait().ok();

        // if below minimum, set to minimum: 2 ticks
        if ticks < 2 {
            ticks = 2;
        }

        self.0
            .start(TimerDuration::<u32, TIMER_HZ>::from_ticks(ticks))
            .unwrap();
        Ok(())
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        match self.0.wait() {
            Ok(()) => {
                /*
                // start another timer to count time between now and next timer
                self.0
                    .start(TimerDuration::<u32, TIMER_HZ>::from_ticks(65535))
                    .unwrap();
                */

                Ok(())
            }
            Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
            Err(nb::Error::Other(_)) => {
                unreachable!("Caught error from infallible method")
            }
        }
    }
}

pub struct StepperTicks<const TIMER_HZ: u32>(pub TimerDuration<u32, TIMER_HZ>);

macro_rules! impl_embedded_time_conversions {
    ($($duration:ident,)*) => {
        $(
            impl<const TIMER_HZ: u32> TryFrom<embedded_time::duration::$duration>
                for StepperTicks<TIMER_HZ>
            {
                type Error = ConversionError;

                fn try_from(duration: embedded_time::duration::$duration)
                    -> Result<Self, Self::Error>
                {
                    let ticks = duration.into_ticks::<u32>(Fraction::new(1, TIMER_HZ))?;
                    Ok(Self(TimerDuration::<u32, TIMER_HZ>::from_ticks(ticks)))
                }
            }
        )*
    };
}

impl_embedded_time_conversions!(
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
);

impl<const TIMER_HZ: u32> ops::Sub for StepperTicks<TIMER_HZ> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        StepperTicks(self.0 - other.0)
    }
}

impl<const TIMER_HZ: u32> defmt::Format for StepperTicks<TIMER_HZ> {
    fn format(&self, f: defmt::Formatter) {
        self.0.format(f)
    }
}
