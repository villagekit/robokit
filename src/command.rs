use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use fugit_timer::Timer;
use heapless::Deque;
use stm32f7xx_hal::{
    gpio::{self, Alternate, Floating, Input, Output, Pin, PushPull},
    pac,
    serial::Serial,
    timer::counter::Counter,
};

use crate::actor::{ActorPoll, ActorReceive, ActorSense};
use crate::actuators::axis::{
    Axis, AxisDriverDQ542MA, AxisDriverErrorDQ542MA, AxisError, AxisLimitMessage, AxisLimitSide,
    AxisLimitStatus, AxisMoveMessage,
};
use crate::actuators::led::{Led, LedBlinkMessage, LedError};
use crate::actuators::spindle::{
    Spindle, SpindleDriver, SpindleDriverJmcHsv57, SpindleError, SpindleSetMessage,
};
use crate::sensors::switch::{Switch, SwitchActiveHigh, SwitchError, SwitchStatus};
use crate::timer::{SubTimer, TICK_TIMER_HZ};

/* actuators */

#[derive(Clone, Copy, Debug, Format)]
pub enum Command {
    GreenLed(LedBlinkMessage<TICK_TIMER_HZ>),
    BlueLed(LedBlinkMessage<TICK_TIMER_HZ>),
    RedLed(LedBlinkMessage<TICK_TIMER_HZ>),
    XAxis(AxisMoveMessage),
    MainSpindle(SpindleSetMessage),
}

pub struct CommandCenterLeds<
    const TIMER_HZ: u32,
    GreenLed: Led<TIMER_HZ>,
    BlueLed: Led<TIMER_HZ>,
    RedLed: Led<TIMER_HZ>,
> {
    pub green_led: GreenLed,
    pub blue_led: BlueLed,
    pub red_led: RedLed,
}

pub struct CommandCenterAxes<XAxis: Axis> {
    pub x_axis: XAxis,
}

pub struct CommandCenterSpindles<MainSpindle: Spindle> {
    pub main_spindle: MainSpindle,
}

#[derive(Debug)]
pub enum ActuatorError<
    const LED_TIMER_HZ: u32,
    GreenLed: Led<LED_TIMER_HZ>,
    BlueLed: Led<LED_TIMER_HZ>,
    RedLed: Led<LED_TIMER_HZ>,
    XAxis: Axis,
    MainSpindle: Spindle,
> {
    GreenLed(<GreenLed as ActorPoll>::Error),
    BlueLed(<BlueLed as ActorPoll>::Error),
    RedLed(<RedLed as ActorPoll>::Error),
    XAxis(<XAxis as ActorPoll>::Error),
    MainSpindle(<MainSpindle as ActorPoll>::Error),
}

pub struct CommandCenterLimitSwitches<XAxisLimitMin: Switch, XAxisLimitMax: Switch> {
    pub x_axis_limit_min: XAxisLimitMin,
    pub x_axis_limit_max: XAxisLimitMax,
}

pub trait CommandCenterTrait:
    ActorReceive<ResetMessage> + ActorReceive<Command> + ActorPoll
{
}

pub struct CommandCenter<
    const LED_TIMER_HZ: u32,
    GreenLed: Led<LED_TIMER_HZ>,
    BlueLed: Led<LED_TIMER_HZ>,
    RedLed: Led<LED_TIMER_HZ>,
    XAxis: Axis,
    MainSpindle: Spindle,
> {
    active_commands: Deque<Command, 8>,
    leds: CommandCenterLeds<LED_TIMER_HZ, GreenLed, BlueLed, RedLed>,
    axes: CommandCenterAxes<XAxis>,
    spindles: CommandCenterSpindles<MainSpindle>,
}

impl<
        const LED_TIMER_HZ: u32,
        GreenLed: Led<LED_TIMER_HZ>,
        BlueLed: Led<LED_TIMER_HZ>,
        RedLed: Led<LED_TIMER_HZ>,
        XAxis: Axis,
        MainSpindle: Spindle,
    > CommandCenter<LED_TIMER_HZ, GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
    pub fn new(
        leds: CommandCenterLeds<LED_TIMER_HZ, GreenLed, BlueLed, RedLed>,
        axes: CommandCenterAxes<XAxis>,
        spindles: CommandCenterSpindles<MainSpindle>,
    ) -> Self {
        Self {
            active_commands: Deque::new(),
            leds,
            axes,
            spindles,
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct ResetMessage {}

impl<const LED_TIMER_HZ: u32, GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
    ActorReceive<ResetMessage>
    for CommandCenter<LED_TIMER_HZ, GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
    fn receive(&mut self, _message: &ResetMessage) {
        self.active_commands.clear()
    }
}

impl<const LED_TIMER_HZ: u32, GreenLed, BlueLed, RedLed, XAxis, MainSpindle> ActorReceive<Command>
    for CommandCenter<LED_TIMER_HZ, GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
    fn receive(&mut self, command: &Command) {
        match command {
            Command::GreenLed(message) => {
                self.actuators.green_led.receive(message);
            }
            Command::BlueLed(message) => {
                self.actuators.blue_led.receive(message);
            }
            Command::RedLed(message) => {
                self.actuators.red_led.receive(message);
            }
            Command::XAxis(message) => {
                self.actuators.x_axis.receive(message);
            }
            Command::MainSpindle(message) => {
                self.actuators.main_spindle.receive(message);
            }
        }

        self.active_commands.push_back(*command).unwrap();
    }
}

impl<const LED_TIMER_HZ: u32, GreenLed, BlueLed, RedLed, XAxis, MainSpindle> ActorPoll
    for CommandCenter<LED_TIMER_HZ, GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
    type Error = ActuatorError<LED_TIMER_HZ, GreenLed, BlueLed, RedLed, XAxis, MainSpindle>;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        let num_commands = self.active_commands.len();
        for _command_index in 0..num_commands {
            let command = self.active_commands.pop_front().unwrap();
            let result = match command {
                Command::GreenLed(_) => self
                    .actuators
                    .green_led
                    .poll()
                    .map_err(|err| ActuatorError::GreenLed(err)),
                Command::BlueLed(_) => self
                    .actuators
                    .blue_led
                    .poll()
                    .map_err(|err| ActuatorError::BlueLed(err)),
                Command::RedLed(_) => self
                    .actuators
                    .red_led
                    .poll()
                    .map_err(|err| ActuatorError::RedLed(err)),
                Command::XAxis(_) => self
                    .actuators
                    .x_axis
                    .poll()
                    .map_err(|err| ActuatorError::XAxis(err)),
                Command::MainSpindle(_) => self
                    .actuators
                    .main_spindle
                    .poll()
                    .map_err(|err| ActuatorError::MainSpindle(err)),
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
