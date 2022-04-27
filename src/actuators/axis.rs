use core::fmt::Debug;
use core::marker::PhantomData;
use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
use stepper::{
    compat, drivers,
    embedded_time::{duration::Nanoseconds, TimeInt},
    motion_control::{self, SoftwareMotionControl},
    ramp_maker,
    traits::{MotionControl, SetDirection, Step},
    Direction, Stepper,
};

use crate::actor::{ActorPoll, ActorReceive};

pub type AxisMotionProfile = ramp_maker::Trapezoidal<f64>;
pub type AxisMotionControl<Driver, Timer, const FREQ: u32> = SoftwareMotionControl<
    Driver,
    compat::Timer<Timer, FREQ>,
    AxisMotionProfile,
    DelayToTicks<<Timer as CountDown>::Time, FREQ>,
>;

pub type AxisDriverDQ542MA<PinDir, PinStep, T, const FREQ: u32> = AxisMotionControl<
    drivers::dq542ma::DQ542MA<(), compat::Pin<PinStep>, compat::Pin<PinDir>>,
    T,
    FREQ,
>;
pub type AxisDriverErrorDQ542MA<PinDir, PinStep, T, const FREQ: u32> =
    <AxisDriverDQ542MA<PinDir, PinStep, T, FREQ> as MotionControl>::Error;

// https://docs.rs/stepper/latest/src/stepper/stepper/move_to.rs.html
pub enum AxisState<Velocity> {
    Idle,
    Initial {
        max_velocity_in_steps_per_sec: Velocity,
        target_step: i32,
    },
    Moving,
}

pub struct Axis<Driver>
where
    Driver: MotionControl,
{
    stepper: Stepper<Driver>,
    steps_per_millimeter: f64,
    state: AxisState<<Driver as MotionControl>::Velocity>,
    logical_position: f64,
}

impl<PinDir, PinStep, T, const FREQ: u32> Axis<AxisDriverDQ542MA<PinDir, PinStep, T, FREQ>>
where
    PinDir: OutputPin,
    <PinDir as OutputPin>::Error: Debug,
    PinStep: OutputPin,
    <PinStep as OutputPin>::Error: Debug,
    T: CountDown,
    <T as CountDown>::Time: TimeInt + From<Nanoseconds>,
    <AxisDriverDQ542MA<PinDir, PinStep, T, FREQ> as MotionControl>::Error: Debug,
{
    pub fn new_dq542ma(
        dir: PinDir,
        step: PinStep,
        timer: T,
        max_acceleration_in_millimeters_per_sec_per_sec: f64,
        steps_per_millimeter: f64,
    ) -> Self {
        let max_acceleration_in_steps_per_sec_per_sec =
            max_acceleration_in_millimeters_per_sec_per_sec * steps_per_millimeter;
        let profile = ramp_maker::Trapezoidal::new(max_acceleration_in_steps_per_sec_per_sec);

        let compat_dir = compat::Pin(dir);
        let compat_step = compat::Pin(step);
        let mut compat_timer = compat::Timer(timer);

        let stepper = Stepper::from_driver(drivers::dq542ma::DQ542MA::new())
            .enable_direction_control(compat_dir, Direction::Forward, &mut compat_timer)
            .unwrap()
            .enable_step_control(compat_step)
            .enable_motion_control((compat_timer, profile, DelayToTicks::new()));

        Axis {
            stepper: stepper,
            steps_per_millimeter,
            state: AxisState::Idle,
            logical_position: 0.,
        }
    }
}

impl<Driver, Timer, const FREQ: u32> Axis<AxisMotionControl<Driver, Timer, FREQ>>
where
    Driver: SetDirection + Step,
    Timer: CountDown,
    Timer::Time: TimeInt + From<Nanoseconds>,
{
    pub fn get_real_position(&mut self) -> f64 {
        (self.stepper.driver_mut().current_step() as f64) / self.steps_per_millimeter
    }
}

pub struct DelayToTicks<Time, const FREQ: u32>(PhantomData<Time>);

impl<Time, const FREQ: u32> DelayToTicks<Time, FREQ> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Time, const FREQ: u32> motion_control::DelayToTicks<f64> for DelayToTicks<Time, FREQ>
where
    Time: From<Nanoseconds>,
{
    type Ticks = compat::Ticks<Time, FREQ>;
    type Error = core::convert::Infallible;

    fn delay_to_ticks(&self, delay: f64) -> Result<Self::Ticks, Self::Error> {
        Ok(compat::Ticks(Nanoseconds(delay as u32).into()))
    }
}

#[derive(Format)]
pub struct AxisMoveMessage {
    pub max_velocity_in_millimeters_per_sec: f64,
    pub distance_in_millimeters: f64,
}

impl<Driver, Timer, const FREQ: u32> ActorReceive for Axis<AxisMotionControl<Driver, Timer, FREQ>>
where
    Driver: SetDirection + Step,
    Timer: CountDown,
    Timer::Time: TimeInt + From<Nanoseconds>,
{
    type Message = AxisMoveMessage;

    fn receive(&mut self, action: &Self::Message) {
        let max_velocity_in_steps_per_sec =
            action.max_velocity_in_millimeters_per_sec * self.steps_per_millimeter;

        let distance_in_millimeters = action.distance_in_millimeters;

        let next_logical_position = self.logical_position + distance_in_millimeters;
        let real_position_difference = next_logical_position - self.get_real_position();
        let step_difference: i32 = (real_position_difference * self.steps_per_millimeter) as i32;
        let target_step = self.stepper.driver_mut().current_step() + step_difference;

        self.state = AxisState::Initial {
            max_velocity_in_steps_per_sec,
            target_step,
        };
    }
}

#[derive(Debug)]
pub enum AxisError<DriverError: Debug> {
    Driver(DriverError),
    Programmer,
}

// https://docs.rs/stepper/latest/src/stepper/stepper/move_to.rs.html#
impl<Driver> ActorPoll for Axis<Driver>
where
    Driver: MotionControl,
    <Driver as MotionControl>::Error: Debug,
{
    type Error = AxisError<<Driver as MotionControl>::Error>;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
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