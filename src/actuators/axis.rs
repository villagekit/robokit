use core::fmt::Debug;
use core::marker::PhantomData;
use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
use ramp_maker;
use stepper::{
    compat, drivers,
    embedded_time::{
        duration::{Duration, Nanoseconds},
        TimeInt,
    },
    motion_control::{self, SoftwareMotionControl},
    traits::{EnableDirectionControl, EnableMotionControl, EnableStepControl, MotionControl},
    util::ref_mut::RefMut,
    Direction, Error as StepperError, MoveToFuture, Stepper,
};

use crate::actor::{ActorPoll, ActorReceive};

type Driver<PinDir, PinStep, T, const FREQ: u32> = SoftwareMotionControl<
    drivers::dq542ma::DQ542MA<(), compat::Pin<PinStep>, compat::Pin<PinDir>>,
    compat::Timer<T, FREQ>,
    ramp_maker::Trapezoidal<f64>,
    DelayToTicks<<T as CountDown>::Time, FREQ>,
>;

pub struct Axis<PinDir, PinStep, T, const FREQ: u32>
where
    PinDir: OutputPin,
    <PinDir as OutputPin>::Error: Debug,
    PinStep: OutputPin,
    <PinStep as OutputPin>::Error: Debug,
    T: CountDown,
    <T as CountDown>::Time: Duration + TimeInt + From<Nanoseconds>,
{
    stepper: Stepper<Driver<PinDir, PinStep, T, FREQ>>,
    move_to_future: Option<MoveToFuture<RefMut<Driver<PinDir, PinStep, T, FREQ>>>>,
    real_position: f64,
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
    <T as CountDown>::Time: Duration + TimeInt + From<Nanoseconds>,
{
    pub fn new(dir: PinDir, step: PinStep, timer: T) -> Self {
        let target_accel = 0.001_f64; // steps / tick^2; 1000 steps / s^2

        let profile = ramp_maker::Trapezoidal::new(target_accel);

        let compat_dir = compat::Pin(dir);
        let compat_step = compat::Pin(step);
        let compat_timer = compat::Timer(timer);

        let stepper = Stepper::from_driver(drivers::dq542ma::DQ542MA::new())
            .enable_direction_control(compat_dir, Direction::Forward, &mut compat_timer)
            .unwrap()
            .enable_step_control(compat_step)
            .enable_motion_control((compat_timer, profile, DelayToTicks::new()));

        Axis {
            stepper: stepper,
            move_to_future: None,
            real_position: 0.,
            logical_position: 0.,
            pin_dir: PhantomData,
            pin_step: PhantomData,
            timer: PhantomData,
        }
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
    Time: Duration + From<Nanoseconds>,
{
    type Ticks = compat::Ticks<Time, FREQ>;
    type Error = core::convert::Infallible;

    fn delay_to_ticks(&self, delay: f64) -> Result<Self::Ticks, Self::Error> {
        Ok(compat::Ticks(Nanoseconds(delay as u32).into()))
    }
}

#[derive(Format)]
pub struct AxisMoveMessage {
    pub distance_in_millimeters: f64,
}

impl<PinDir, PinStep, T, const FREQ: u32> ActorReceive for Axis<PinDir, PinStep, T, FREQ>
where
    PinDir: OutputPin,
    <PinDir as OutputPin>::Error: Debug,
    PinStep: OutputPin,
    <PinStep as OutputPin>::Error: Debug,
    T: CountDown,
    <T as CountDown>::Time: Duration + TimeInt + From<Nanoseconds>,
{
    type Message = AxisMoveMessage;

    fn receive(&mut self, action: &Self::Message) {
        // TODO
        // convert millimeters to steps
        // update next logical position
        // calculate steps to move from current actual position to next logical position

        let max_speed = 0.001_f64; // steps / tick; 1000 steps / s
        let target_step = 10;
        self.move_to_future = Some(self.stepper.move_to_position(max_speed, target_step));
    }
}

#[derive(Debug)]
pub enum AxisError {
    Stepper,
    Unknown,
}

impl<PinDir, PinStep, T, const FREQ: u32> ActorPoll for Axis<PinDir, PinStep, T, FREQ>
where
    PinDir: OutputPin,
    <PinDir as OutputPin>::Error: Debug,
    PinStep: OutputPin,
    <PinStep as OutputPin>::Error: Debug,
    T: CountDown,
    <T as CountDown>::Time: Duration + TimeInt + From<Nanoseconds>,
{
    type Error = AxisError;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        if let Some(move_to_future) = self.move_to_future {
            move_to_future.poll().map_err(|_err| AxisError::Stepper)
        } else {
            Poll::Ready(Err(AxisError::Unknown))
        }
    }
}
