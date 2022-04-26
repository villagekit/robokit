use core::fmt::Debug;
use core::marker::PhantomData;
use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
use ramp_maker;
use stepper::{
    compat, drivers,
    embedded_time::{duration::Nanoseconds, TimeInt},
    motion_control::{self, SoftwareMotionControl},
    traits::MotionControl,
    Direction, Stepper,
};

use crate::actor::{ActorPoll, ActorReceive};

pub type Driver<PinDir, PinStep, T, const FREQ: u32> = SoftwareMotionControl<
    drivers::dq542ma::DQ542MA<(), compat::Pin<PinStep>, compat::Pin<PinDir>>,
    compat::Timer<T, FREQ>,
    ramp_maker::Trapezoidal<f64>,
    DelayToTicks<<T as CountDown>::Time, FREQ>,
>;
pub type DriverError<PinDir, PinStep, T, const FREQ: u32> =
    <Driver<PinDir, PinStep, T, FREQ> as MotionControl>::Error;

// https://docs.rs/stepper/latest/src/stepper/stepper/move_to.rs.html
pub enum AxisState<Velocity> {
    Idle,
    Initial {
        max_velocity: Velocity,
        target_step: i32,
    },
    Moving,
}

pub struct Axis<PinDir, PinStep, T, const FREQ: u32>
where
    PinDir: OutputPin,
    <PinDir as OutputPin>::Error: Debug,
    PinStep: OutputPin,
    <PinStep as OutputPin>::Error: Debug,
    T: CountDown,
    <T as CountDown>::Time: TimeInt + From<Nanoseconds>,
    <Driver<PinDir, PinStep, T, FREQ> as MotionControl>::Error: Debug,
{
    stepper: Stepper<Driver<PinDir, PinStep, T, FREQ>>,
    steps_per_millimeter: f64,
    state: AxisState<<Driver<PinDir, PinStep, T, FREQ> as MotionControl>::Velocity>,
    logical_position: f64,
    pin_dir: PhantomData<PinDir>,
    pin_step: PhantomData<PinStep>,
    timer: PhantomData<T>,
}

impl<PinDir, PinStep, T, const FREQ: u32> Axis<PinDir, PinStep, T, FREQ>
where
    PinDir: OutputPin,
    <PinDir as OutputPin>::Error: Debug,
    PinStep: OutputPin,
    <PinStep as OutputPin>::Error: Debug,
    T: CountDown,
    <T as CountDown>::Time: TimeInt + From<Nanoseconds>,
    <Driver<PinDir, PinStep, T, FREQ> as MotionControl>::Error: Debug,
{
    pub fn new(
        dir: PinDir,
        step: PinStep,
        timer: T,
        max_accel: f64,
        steps_per_millimeter: f64,
    ) -> Self {
        let profile = ramp_maker::Trapezoidal::new(max_accel);

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
            pin_dir: PhantomData,
            pin_step: PhantomData,
            timer: PhantomData,
        }
    }

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
    pub max_velocity: f64,
    pub distance_in_millimeters: f64,
}

impl<PinDir, PinStep, T, const FREQ: u32> ActorReceive for Axis<PinDir, PinStep, T, FREQ>
where
    PinDir: OutputPin,
    <PinDir as OutputPin>::Error: Debug,
    PinStep: OutputPin,
    <PinStep as OutputPin>::Error: Debug,
    T: CountDown,
    <T as CountDown>::Time: TimeInt + From<Nanoseconds>,
    <Driver<PinDir, PinStep, T, FREQ> as MotionControl>::Error: Debug,
{
    type Message = AxisMoveMessage;

    fn receive(&mut self, action: &Self::Message) {
        let max_velocity = action.max_velocity;
        let distance_in_millimeters = action.distance_in_millimeters;

        let next_logical_position = self.logical_position + distance_in_millimeters;
        let real_position_difference = next_logical_position - self.get_real_position();
        let step_difference: i32 = (real_position_difference * self.steps_per_millimeter) as i32;
        let target_step = self.stepper.driver_mut().current_step() + step_difference;

        self.state = AxisState::Initial {
            max_velocity,
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
impl<PinDir, PinStep, T, const FREQ: u32> ActorPoll for Axis<PinDir, PinStep, T, FREQ>
where
    PinDir: OutputPin,
    <PinDir as OutputPin>::Error: Debug,
    PinStep: OutputPin,
    <PinStep as OutputPin>::Error: Debug,
    T: CountDown,
    <T as CountDown>::Time: TimeInt + From<Nanoseconds>,
    <Driver<PinDir, PinStep, T, FREQ> as MotionControl>::Error: Debug,
{
    type Error = AxisError<DriverError<PinDir, PinStep, T, FREQ>>;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        match self.state {
            AxisState::Idle => Poll::Ready(Ok(())),
            AxisState::Initial {
                max_velocity,
                target_step,
            } => {
                self.stepper
                    .driver_mut()
                    .move_to_position(max_velocity, target_step)
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
