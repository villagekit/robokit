use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use heapless::Deque;

use crate::actor::{ActorPoll, ActorReceive};
use crate::actuators::axis::{Axis, AxisMoveMessage};
use crate::actuators::led::{Led, LedBlinkMessage};
use crate::actuators::spindle::{Spindle, SpindleSetMessage};
use crate::timer::TICK_TIMER_HZ;

type PollError<T> = <T as ActorPoll>::Error;

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
    GreenLed: Led<TICK_TIMER_HZ>,
    BlueLed: Led<TICK_TIMER_HZ>,
    RedLed: Led<TICK_TIMER_HZ>,
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
pub enum CommandError<
    GreenLedError: Debug,
    BlueLedError: Debug,
    RedLedError: Debug,
    XAxisError: Debug,
    MainSpindleError: Debug,
> {
    GreenLed(GreenLedError),
    BlueLed(BlueLedError),
    RedLed(RedLedError),
    XAxis(XAxisError),
    MainSpindle(MainSpindleError),
}

pub trait CommandCenterTrait:
    ActorReceive<ResetMessage> + ActorReceive<Command> + ActorPoll
{
}

pub struct CommandCenter<
    GreenLed: Led<TICK_TIMER_HZ>,
    BlueLed: Led<TICK_TIMER_HZ>,
    RedLed: Led<TICK_TIMER_HZ>,
    XAxis: Axis,
    MainSpindle: Spindle,
> {
    active_commands: Deque<Command, 8>,
    leds: CommandCenterLeds<GreenLed, BlueLed, RedLed>,
    axes: CommandCenterAxes<XAxis>,
    spindles: CommandCenterSpindles<MainSpindle>,
}

impl<
        GreenLed: Led<TICK_TIMER_HZ>,
        BlueLed: Led<TICK_TIMER_HZ>,
        RedLed: Led<TICK_TIMER_HZ>,
        XAxis: Axis,
        MainSpindle: Spindle,
    > CommandCenterTrait for CommandCenter<GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
}

impl<
        GreenLed: Led<TICK_TIMER_HZ>,
        BlueLed: Led<TICK_TIMER_HZ>,
        RedLed: Led<TICK_TIMER_HZ>,
        XAxis: Axis,
        MainSpindle: Spindle,
    > CommandCenter<GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
    pub fn new(
        leds: CommandCenterLeds<GreenLed, BlueLed, RedLed>,
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

impl<
        GreenLed: Led<TICK_TIMER_HZ>,
        BlueLed: Led<TICK_TIMER_HZ>,
        RedLed: Led<TICK_TIMER_HZ>,
        XAxis: Axis,
        MainSpindle: Spindle,
    > ActorReceive<ResetMessage> for CommandCenter<GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
    fn receive(&mut self, _message: &ResetMessage) {
        self.active_commands.clear()
    }
}

impl<
        GreenLed: Led<TICK_TIMER_HZ>,
        BlueLed: Led<TICK_TIMER_HZ>,
        RedLed: Led<TICK_TIMER_HZ>,
        XAxis: Axis,
        MainSpindle: Spindle,
    > ActorReceive<Command> for CommandCenter<GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
    fn receive(&mut self, command: &Command) {
        match command {
            Command::GreenLed(message) => {
                self.leds.green_led.receive(message);
            }
            Command::BlueLed(message) => {
                self.leds.blue_led.receive(message);
            }
            Command::RedLed(message) => {
                self.leds.red_led.receive(message);
            }
            Command::XAxis(message) => {
                self.axes.x_axis.receive(message);
            }
            Command::MainSpindle(message) => {
                self.spindles.main_spindle.receive(message);
            }
        }

        self.active_commands.push_back(*command).unwrap();
    }
}

impl<GreenLed, BlueLed, RedLed, XAxis, MainSpindle> ActorPoll
    for CommandCenter<GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
where
    GreenLed: Led<TICK_TIMER_HZ>,
    BlueLed: Led<TICK_TIMER_HZ>,
    RedLed: Led<TICK_TIMER_HZ>,
    XAxis: Axis,
    MainSpindle: Spindle,
{
    type Error = CommandError<
        PollError<GreenLed>,
        PollError<BlueLed>,
        PollError<RedLed>,
        PollError<XAxis>,
        PollError<MainSpindle>,
    >;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        let num_commands = self.active_commands.len();
        for _command_index in 0..num_commands {
            let command = self.active_commands.pop_front().unwrap();
            let result = match command {
                Command::GreenLed(_) => self
                    .leds
                    .green_led
                    .poll()
                    .map_err(|err| CommandError::GreenLed(err)),
                Command::BlueLed(_) => self
                    .leds
                    .blue_led
                    .poll()
                    .map_err(|err| CommandError::BlueLed(err)),
                Command::RedLed(_) => self
                    .leds
                    .red_led
                    .poll()
                    .map_err(|err| CommandError::RedLed(err)),
                Command::XAxis(_) => self
                    .axes
                    .x_axis
                    .poll()
                    .map_err(|err| CommandError::XAxis(err)),
                Command::MainSpindle(_) => self
                    .spindles
                    .main_spindle
                    .poll()
                    .map_err(|err| CommandError::MainSpindle(err)),
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
