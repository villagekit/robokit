use core::task::Poll;

use alloc::boxed::Box;
use defmt::Format;
use robokit::{
    actuators::{
        axis::{AnyAxis, AxisAction},
        led::{AnyLed, LedAction},
        spindle::{AnySpindle, SpindleAction},
    },
    error::Error,
    runner::ActuatorSet,
    timer::TICK_TIMER_HZ,
};

#[derive(Clone, Copy, Debug, Format)]
pub enum LedId {
    Green,
    Blue,
    Red,
}

pub struct LedSet<
    GreenLed: AnyLed<TICK_TIMER_HZ>,
    BlueLed: AnyLed<TICK_TIMER_HZ>,
    RedLed: AnyLed<TICK_TIMER_HZ>,
> {
    pub green: GreenLed,
    pub blue: BlueLed,
    pub red: RedLed,
}

impl<GreenLed, BlueLed, RedLed> ActuatorSet for LedSet<GreenLed, BlueLed, RedLed>
where
    GreenLed: AnyLed<TICK_TIMER_HZ>,
    GreenLed::Error: 'static,
    BlueLed: AnyLed<TICK_TIMER_HZ>,
    BlueLed::Error: 'static,
    RedLed: AnyLed<TICK_TIMER_HZ>,
    RedLed::Error: 'static,
{
    type Action = LedAction<TICK_TIMER_HZ>;
    type Id = LedId;

    fn run(&mut self, id: &Self::Id, action: &LedAction<TICK_TIMER_HZ>) {
        match id {
            LedId::Green => self.green.run(action),
            LedId::Blue => self.blue.run(action),
            LedId::Red => self.red.run(action),
        }
    }

    fn poll(&mut self, id: &Self::Id) -> Poll<Result<(), Box<dyn Error>>> {
        match id {
            LedId::Green => self.green.poll().map_err(Into::into),
            LedId::Blue => self.blue.poll().map_err(Into::into),
            LedId::Red => self.red.poll().map_err(Into::into),
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub enum AxisId {
    X,
}

pub struct AxisSet<XAxis: AnyAxis> {
    pub x: XAxis,
}

impl<XAxis> ActuatorSet for AxisSet<XAxis>
where
    XAxis: AnyAxis,
    XAxis::Error: 'static,
{
    type Action = AxisAction;
    type Id = AxisId;

    fn run(&mut self, id: &Self::Id, action: &AxisAction) {
        match id {
            AxisId::X => self.x.run(action),
        }
    }

    fn poll(&mut self, id: &Self::Id) -> Poll<Result<(), Box<dyn Error>>> {
        match id {
            AxisId::X => self.x.poll().map_err(Into::into),
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub enum SpindleId {
    Main,
}

pub struct SpindleSet<MainSpindle: AnySpindle> {
    pub main: MainSpindle,
}

impl<MainSpindle> ActuatorSet for SpindleSet<MainSpindle>
where
    MainSpindle: AnySpindle,
    MainSpindle::Error: 'static,
{
    type Action = SpindleAction;
    type Id = SpindleId;

    fn run(&mut self, id: &Self::Id, action: &SpindleAction) {
        match id {
            SpindleId::Main => self.main.run(action),
        }
    }

    fn poll(&mut self, id: &Self::Id) -> Poll<Result<(), Box<dyn Error>>> {
        match id {
            SpindleId::Main => self.main.poll().map_err(Into::into),
        }
    }
}
