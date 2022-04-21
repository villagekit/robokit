use core::fmt::Debug;
use core::marker::PhantomData;
use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
use embedded_time::duration::Nanoseconds;
use fugit::TimerDurationU32;
use ramp_maker;
use stepper::{
    compat, drivers,
    motion_control::{self, SoftwareMotionControl},
    traits::{
        EnableDirectionControl, EnableMotionControl, EnableStepControl, MotionControl,
        SetDirection, Step,
    },
    Direction, Error as StepperError, MoveToFuture, Stepper,
};

use crate::actor::{ActorPoll, ActorReceive};

type Driver<PinDir, PinStep, T> = SoftwareMotionControl<
    drivers::dq542ma::DQ542MA<(), compat::Pin<PinDir>, compat::Pin<PinStep>>,
    compat::Timer<T, 1_000_000>,
    ramp_maker::Trapezoidal<f64>,
    DelayToTicks,
>;

pub struct Axis<PinDir, PinStep, T>
where
    PinDir: OutputPin,
    <PinDir as OutputPin>::Error: Debug,
    PinStep: OutputPin,
    <PinStep as OutputPin>::Error: Debug,
    T: CountDown,
{
    stepper: Stepper<Driver<PinDir, PinStep, T>>,
    move_to_future: Option<MoveToFuture<Driver<PinDir, PinStep, T>>>,
    real_position: f64,
    logical_position: f64,
    pin_dir: PhantomData<PinDir>,
    pin_step: PhantomData<PinStep>,
    timer: PhantomData<T>,
}

impl<PinDir, PinStep, T> Axis<PinDir, PinStep, T>
where
    PinDir: OutputPin,
    PinStep: OutputPin,
    T: CountDown,
{
    pub fn new(dir: PinDir, step: PinStep, timer: T) -> Self
    where
        <PinDir as OutputPin>::Error: Debug,
        <PinStep as OutputPin>::Error: Debug,
    {
        let target_accel = 0.001_f64; // steps / tick^2; 1000 steps / s^2
        let max_speed = 0.001_f64; // steps / tick; 1000 steps / s

        let profile = ramp_maker::Trapezoidal::new(target_accel);

        let compat_dir = compat::Pin(dir);
        let compat_step = compat::Pin(step);
        let compat_timer = compat::Timer(timer);

        let stepper = Stepper::from_driver(drivers::dq542ma::DQ542MA::new())
            .enable_direction_control(compat_dir, Direction::Forward, &mut compat_timer)
            .unwrap()
            .enable_step_control(compat_step)
            .enable_motion_control((compat_timer, profile, DelayToTicks));

        Axis {
            stepper,
            move_to_future: None,
            real_position: 0.,
            logical_position: 0.,
            pin_dir: PhantomData,
            pin_step: PhantomData,
            timer: PhantomData,
        }
    }
}

pub struct DelayToTicks;
impl motion_control::DelayToTicks<f64> for DelayToTicks {
    type Ticks = Nanoseconds;
    type Error = core::convert::Infallible;

    fn delay_to_ticks(&self, delay: f64) -> Result<Self::Ticks, Self::Error> {
        Ok(Nanoseconds(delay.to_num::<u32>()))
    }
}

#[derive(Format)]
pub struct AxisMoveMessage {
    pub distance_in_millimeters: f64,
}

impl<PinDir, PinStep, T> ActorReceive for Axis<PinDir, PinStep, T>
where
    PinDir: OutputPin,
    PinStep: OutputPin,
    T: CountDown,
{
    type Message = AxisMoveMessage;

    fn receive(&mut self, action: &Self::Message) {
        // TODO
        // convert millimeters to steps
        // update next logical position
        // calculate steps to move from current actual position to next logical position
        self.move_to_future = Some(self.stepper.move_to_position(10));
    }
}

#[derive(Debug)]
pub enum AxisError {
    Stepper,
}

impl<PinDir, PinStep, T> ActorPoll for Axis<PinDir, PinStep, T>
where
    PinDir: OutputPin,
    PinStep: OutputPin,
    T: CountDown,
{
    type Error = AxisError;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        if let Some(move_to_future) = self.move_to_future {
            move_to_future.poll()
        }
    }
}
