use core::marker::PhantomData;
use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use fixed;
use fugit::MillisDurationU32 as MillisDuration;
use fugit_timer::Timer;
use stepper::{
    drivers, motion_control, ramp_maker,
    traits::{
        EnableDirectionControl, EnableMotionControl, EnableStepControl, MotionControl,
        SetDirection, Step,
    },
    Direction, Error as StepperError, MoveToFuture, Stepper,
};
use typenum;

use crate::actor::{ActorPoll, ActorReceive};

pub struct Axis<Driver, PinDir, PinStep, T>
where
    Driver: EnableDirectionControl<PinDir> + EnableStepControl<PinStep> + MotionControl,
    PinDir: OutputPin,
    PinStep: OutputPin,
    T: Timer<1_000_000>,
{
    stepper: Stepper<Driver>,
    move_to_future: Option<MoveToFuture<Driver>>,
    real_position: f64,
    logical_position: f64,
    pin_dir: PhantomData<PinDir>,
    pin_step: PhantomData<PinStep>,
    timer: PhantomData<T>,
}

type Num = fixed::FixedI64<typenum::U32>;

impl<Driver, PinDir, PinStep, T> Axis<Driver, PinDir, PinStep, T>
where
    Driver: EnableDirectionControl<PinDir> + EnableStepControl<PinStep> + MotionControl,
    PinDir: OutputPin,
    PinStep: OutputPin,
    T: Timer<1_000_000>,
{
    pub fn new(dir: PinDir, step: PinStep, timer: T) -> Self {
        let target_accel = Num::from_num(0.001); // steps / tick^2; 1000 steps / s^2
        let max_speed = Num::from_num(0.001); // steps / tick; 1000 steps / s

        let profile = ramp_maker::Trapezoidal::new(target_accel);

        let stepper = Stepper::from_driver(drivers::dq542ma::DQ542MA::new())
            .enable_direction_control(dir, Direction::Forward, &mut timer)?
            .enable_step_control(step)
            .enable_motion_control((timer, profile, DelayToTicks));

        Axis {
            stepper,
            move_to_future: None,
            real_position: 0.,
            logical_position: 0.,
        }
    }
}

pub struct DelayToTicks;
impl motion_control::DelayToTicks<Num> for DelayToTicks {
    type Ticks = MillisDuration;
    type Error = core::convert::Infallible;

    fn delay_to_ticks(&self, delay: Num) -> Result<Self::Ticks, Self::Error> {
        Ok(MillisDuration::from_ticks(delay.int()))
    }
}

#[derive(Format)]
pub struct AxisMoveMessage {
    pub distance_in_millimeters: f64,
}

impl<Driver, PinDir, PinStep, T> ActorReceive for Axis<Driver, PinDir, PinStep, T>
where
    Driver: EnableDirectionControl<PinDir> + EnableStepControl<PinStep> + MotionControl,
    PinDir: OutputPin,
    PinStep: OutputPin,
    T: Timer<1_000_000>,
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

impl<Driver, PinDir, PinStep, T> ActorPoll for Axis<Driver, PinDir, PinStep, T>
where
    Driver: EnableDirectionControl<PinDir> + EnableStepControl<PinStep> + MotionControl,
    PinDir: OutputPin,
    PinStep: OutputPin,
    T: Timer<1_000_000>,
{
    type Error = AxisError;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        if let Some(move_to_future) = self.move_to_future {
            move_to_future.poll()
        }
    }
}
