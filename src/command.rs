use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use heapless::Deque;

use crate::actor::{ActorPoll, ActorReceive};
use crate::actuators::axis::{
    AnyAxis, AxisHomeMessage, AxisMoveAbsoluteMessage, AxisMoveRelativeMessage,
};
use crate::actuators::led::{AnyLed, LedBlinkMessage};
use crate::actuators::spindle::{AnySpindle, SpindleSetMessage};
use crate::timer::TICK_TIMER_HZ;

pub trait AnyCommandCenter: ActorReceive<ResetMessage> + ActorReceive<Command> + ActorPoll {}

/* actuators */

#[derive(Clone, Copy, Debug, Format)]
pub enum Command {
    GreenLedBlink(LedBlinkMessage<TICK_TIMER_HZ>),
    BlueLedBlink(LedBlinkMessage<TICK_TIMER_HZ>),
    RedLedBlink(LedBlinkMessage<TICK_TIMER_HZ>),
    XAxisMoveRelative(AxisMoveRelativeMessage),
    XAxisMoveAbsolute(AxisMoveAbsoluteMessage),
    XAxisHome(AxisHomeMessage),
    MainSpindleSet(SpindleSetMessage),
}

pub struct CommandCenterLeds<
    GreenLed: AnyLed<TICK_TIMER_HZ>,
    BlueLed: AnyLed<TICK_TIMER_HZ>,
    RedLed: AnyLed<TICK_TIMER_HZ>,
> {
    pub green_led: GreenLed,
    pub blue_led: BlueLed,
    pub red_led: RedLed,
}

pub struct CommandCenterAxes<XAxis: AnyAxis> {
    pub x_axis: XAxis,
}

pub struct CommandCenterSpindles<MainSpindle: AnySpindle> {
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

pub struct CommandCenter<
    GreenLed: AnyLed<TICK_TIMER_HZ>,
    BlueLed: AnyLed<TICK_TIMER_HZ>,
    RedLed: AnyLed<TICK_TIMER_HZ>,
    XAxis: AnyAxis,
    MainSpindle: AnySpindle,
> {
    active_commands: Deque<Command, 8>,
    leds: CommandCenterLeds<GreenLed, BlueLed, RedLed>,
    axes: CommandCenterAxes<XAxis>,
    spindles: CommandCenterSpindles<MainSpindle>,
}

impl<
        GreenLed: AnyLed<TICK_TIMER_HZ>,
        BlueLed: AnyLed<TICK_TIMER_HZ>,
        RedLed: AnyLed<TICK_TIMER_HZ>,
        XAxis: AnyAxis,
        MainSpindle: AnySpindle,
    > AnyCommandCenter for CommandCenter<GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
}

impl<
        GreenLed: AnyLed<TICK_TIMER_HZ>,
        BlueLed: AnyLed<TICK_TIMER_HZ>,
        RedLed: AnyLed<TICK_TIMER_HZ>,
        XAxis: AnyAxis,
        MainSpindle: AnySpindle,
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
        GreenLed: AnyLed<TICK_TIMER_HZ>,
        BlueLed: AnyLed<TICK_TIMER_HZ>,
        RedLed: AnyLed<TICK_TIMER_HZ>,
        XAxis: AnyAxis,
        MainSpindle: AnySpindle,
    > ActorReceive<ResetMessage> for CommandCenter<GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
    fn receive(&mut self, _message: &ResetMessage) {
        self.active_commands.clear()
    }
}

impl<
        GreenLed: AnyLed<TICK_TIMER_HZ>,
        BlueLed: AnyLed<TICK_TIMER_HZ>,
        RedLed: AnyLed<TICK_TIMER_HZ>,
        XAxis: AnyAxis,
        MainSpindle: AnySpindle,
    > ActorReceive<Command> for CommandCenter<GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
{
    fn receive(&mut self, command: &Command) {
        match command {
            Command::GreenLedBlink(message) => {
                self.leds.green_led.receive(message);
            }
            Command::BlueLedBlink(message) => {
                self.leds.blue_led.receive(message);
            }
            Command::RedLedBlink(message) => {
                self.leds.red_led.receive(message);
            }
            Command::XAxisMoveRelative(message) => {
                self.axes.x_axis.receive(message);
            }
            Command::XAxisMoveAbsolute(message) => {
                self.axes.x_axis.receive(message);
            }
            Command::XAxisHome(message) => {
                self.axes.x_axis.receive(message);
            }
            Command::MainSpindleSet(message) => {
                self.spindles.main_spindle.receive(message);
            }
        }

        self.active_commands.push_back(*command).unwrap();
    }
}

type PollError<T> = <T as ActorPoll>::Error;

impl<GreenLed, BlueLed, RedLed, XAxis, MainSpindle> ActorPoll
    for CommandCenter<GreenLed, BlueLed, RedLed, XAxis, MainSpindle>
where
    GreenLed: AnyLed<TICK_TIMER_HZ>,
    BlueLed: AnyLed<TICK_TIMER_HZ>,
    RedLed: AnyLed<TICK_TIMER_HZ>,
    XAxis: AnyAxis,
    MainSpindle: AnySpindle,
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
                Command::GreenLedBlink(_) => self
                    .leds
                    .green_led
                    .poll()
                    .map_err(|err| CommandError::GreenLed(err)),
                Command::BlueLedBlink(_) => self
                    .leds
                    .blue_led
                    .poll()
                    .map_err(|err| CommandError::BlueLed(err)),
                Command::RedLedBlink(_) => self
                    .leds
                    .red_led
                    .poll()
                    .map_err(|err| CommandError::RedLed(err)),
                Command::XAxisMoveRelative(_) => self
                    .axes
                    .x_axis
                    .poll()
                    .map_err(|err| CommandError::XAxis(err)),
                Command::XAxisMoveAbsolute(_) => self
                    .axes
                    .x_axis
                    .poll()
                    .map_err(|err| CommandError::XAxis(err)),
                Command::XAxisHome(_) => self
                    .axes
                    .x_axis
                    .poll()
                    .map_err(|err| CommandError::XAxis(err)),
                Command::MainSpindleSet(_) => self
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
