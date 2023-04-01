use alloc::boxed::Box;
use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use heapless::Deque;

use crate::timer::TICK_TIMER_HZ;
use crate::{
    actuators::{
        axis::{AnyAxis, AxisAction},
        led::{AnyLed, LedAction},
        spindle::{AnySpindle, SpindleAction},
        Actuator,
    },
    error::Error,
};

#[derive(Clone, Copy, Debug, Format)]
pub enum Command {
    Led(LedId, LedAction<TICK_TIMER_HZ>),
    Axis(AxisId, AxisAction),
    Spindle(SpindleId, SpindleAction),
}

#[derive(Clone, Copy, Debug, Format)]
pub enum RunnerAction {
    Run(Command),
    Reset,
}

pub trait AnyRunner: Actuator<RunnerAction> {}

#[derive(Clone, Copy, Debug, Format)]
pub enum LedId {
    Green,
    Blue,
    Red,
}

#[derive(Clone, Copy, Debug, Format)]
pub enum AxisId {
    X,
}

#[derive(Clone, Copy, Debug, Format)]
pub enum SpindleId {
    Main,
}

pub struct RunnerLeds<
    GreenLed: AnyLed<TICK_TIMER_HZ>,
    BlueLed: AnyLed<TICK_TIMER_HZ>,
    RedLed: AnyLed<TICK_TIMER_HZ>,
> {
    pub green_led: GreenLed,
    pub blue_led: BlueLed,
    pub red_led: RedLed,
}

pub struct RunnerAxes<XAxis: AnyAxis> {
    pub x_axis: XAxis,
}

pub struct RunnerSpindles<MainSpindle: AnySpindle> {
    pub main_spindle: MainSpindle,
}

#[derive(Debug)]
pub enum CommandError {
    Led(LedId, Box<dyn Debug>),
    Axis(AxisId, Box<dyn Debug>),
    Spindle(SpindleId, Box<dyn Debug>),
}

pub struct Runner<
    GreenLed: AnyLed<TICK_TIMER_HZ>,
    BlueLed: AnyLed<TICK_TIMER_HZ>,
    RedLed: AnyLed<TICK_TIMER_HZ>,
    XAxis: AnyAxis,
    MainSpindle: AnySpindle,
> {
    active_commands: Deque<Command, 8>,
    leds: RunnerLeds<GreenLed, BlueLed, RedLed>,
    axes: RunnerAxes<XAxis>,
    spindles: RunnerSpindles<MainSpindle>,
}

impl<
        GreenLed: AnyLed<TICK_TIMER_HZ>,
        BlueLed: AnyLed<TICK_TIMER_HZ>,
        RedLed: AnyLed<TICK_TIMER_HZ>,
        XAxis: AnyAxis,
        MainSpindle: AnySpindle,
    > AnyRunner for Runner<GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
}

impl<
        GreenLed: AnyLed<TICK_TIMER_HZ>,
        BlueLed: AnyLed<TICK_TIMER_HZ>,
        RedLed: AnyLed<TICK_TIMER_HZ>,
        XAxis: AnyAxis,
        MainSpindle: AnySpindle,
    > Runner<GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
    pub fn new(
        leds: RunnerLeds<GreenLed, BlueLed, RedLed>,
        axes: RunnerAxes<XAxis>,
        spindles: RunnerSpindles<MainSpindle>,
    ) -> Self {
        Self {
            active_commands: Deque::new(),
            leds,
            axes,
            spindles,
        }
    }
}

impl<
        GreenLed: AnyLed<TICK_TIMER_HZ>,
        BlueLed: AnyLed<TICK_TIMER_HZ>,
        RedLed: AnyLed<TICK_TIMER_HZ>,
        XAxis: AnyAxis,
        MainSpindle: AnySpindle,
    > Actuator<RunnerAction> for Runner<GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
    fn receive(&mut self, action: &RunnerAction) {
        match action {
            RunnerAction::Run(command) => {
                match command {
                    Command::Led(LedId::Green, action) => {
                        self.leds.green_led.receive(action);
                    }
                    Command::Led(LedId::Blue, action) => {
                        self.leds.blue_led.receive(action);
                    }
                    Command::Led(LedId::Red, action) => {
                        self.leds.red_led.receive(action);
                    }
                    Command::Axis(AxisId::X, action) => {
                        self.axes.x_axis.receive(action);
                    }
                    Command::Spindle(SpindleId::Main, action) => {
                        self.spindles.main_spindle.receive(action);
                    }
                }

                self.active_commands.push_back(*command).unwrap();
            }
            RunnerAction::Reset => self.active_commands.clear(),
        }
    }

    fn poll(&mut self) -> Poll<Result<(), Error>> {
        let num_commands = self.active_commands.len();
        for _command_index in 0..num_commands {
            let command = self.active_commands.pop_front().unwrap();
            let result = match command {
                Command::Led(LedId::Green, _) => self
                    .leds
                    .green_led
                    .poll()
                    .map_err(|err| Box::new(CommandError::Led(LedId::Green, err))),
                Command::Led(LedId::Blue, _) => self
                    .leds
                    .blue_led
                    .poll()
                    .map_err(|err| Box::new(CommandError::Led(LedId::Blue, err))),
                Command::Led(LedId::Red, _) => self
                    .leds
                    .red_led
                    .poll()
                    .map_err(|err| Box::new(CommandError::Led(LedId::Red, err))),
                Command::Axis(AxisId::X, _) => self
                    .axes
                    .x_axis
                    .poll()
                    .map_err(|err| Box::new(CommandError::Axis(AxisId::X, err))),
                Command::Spindle(SpindleId::Main, _) => {
                    self.spindles.main_spindle.poll().map_err(|err| {
                        Box::new(CommandError::Spindle(SpindleId::Main, Box::new(err)))
                    })
                }
            };

            match result {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(err)) => {
                    self.active_commands.push_back(command).unwrap();

                    return Poll::Ready(Err(err));
                }
                Poll::Pending => {
                    self.active_commands.push_back(command).unwrap();
                }
            }
        }

        if self.active_commands.len() == 0 {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}
