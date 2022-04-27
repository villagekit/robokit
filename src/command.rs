use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use fugit_timer::Timer;
use stm32f7xx_hal::{
    gpio::{Output, Pin, PushPull},
    pac,
    prelude::*,
    rcc::Clocks,
    timer::{
        counter::{CounterMs, CounterUs},
        /*Error as TimerError,*/ TimerExt,
    },
};

use crate::actor::{ActorPoll, ActorReceive};
use crate::actuators::axis::{
    Axis, AxisDriverDQ542MA, AxisDriverErrorDQ542MA, AxisError, AxisMoveMessage,
};
use crate::actuators::led::{Led, LedBlinkMessage, LedError};
use crate::timer::EmbeddedTimeCounter;

#[derive(Format)]
pub enum Command {
    GreenLed(LedBlinkMessage),
    BlueLed(LedBlinkMessage),
    RedLed(LedBlinkMessage),
    XAxis(AxisMoveMessage),
}

pub enum CommandActor {
    GreenLed,
    BlueLed,
    RedLed,
    XAxis,
}

#[allow(non_snake_case)]
pub struct CommandCenterResources<'a> {
    pub GPIOB: pac::GPIOB,
    pub GPIOG: pac::GPIOG,
    pub TIM3: pac::TIM3,
    pub TIM4: pac::TIM4,
    pub TIM5: pac::TIM5,
    pub TIM6: pac::TIM6,
    pub clocks: &'a Clocks,
}

type GreenLedPin = Pin<'B', 0, Output<PushPull>>;
type GreenLedTimer = CounterMs<pac::TIM3>;
type BlueLedPin = Pin<'B', 7, Output<PushPull>>;
type BlueLedTimer = CounterMs<pac::TIM4>;
type RedLedPin = Pin<'B', 14, Output<PushPull>>;
type RedLedTimer = CounterMs<pac::TIM5>;
type XAxisDirPin = Pin<'G', 9, Output<PushPull>>; // D0
type XAxisStepPin = Pin<'G', 14, Output<PushPull>>; // D1
type XAxisTimer = EmbeddedTimeCounter<CounterUs<pac::TIM6>>;
type XAxisDriver = AxisDriverDQ542MA<XAxisDirPin, XAxisStepPin, XAxisTimer, 1_000_000>;
type XAxisDriverError = AxisDriverErrorDQ542MA<XAxisDirPin, XAxisStepPin, XAxisTimer, 1_000_000>;

pub struct CommandCenterActors {
    pub green_led: Led<GreenLedPin, GreenLedTimer>,
    pub blue_led: Led<BlueLedPin, BlueLedTimer>,
    pub red_led: Led<RedLedPin, RedLedTimer>,
    pub x_axis: Axis<XAxisDriver>,
}

#[derive(Debug)]
pub enum CommandError {
    GreenLed(LedError<<GreenLedPin as OutputPin>::Error, <GreenLedTimer as Timer<1_000>>::Error>),
    BlueLed(LedError<<BlueLedPin as OutputPin>::Error, <BlueLedTimer as Timer<1_000>>::Error>),
    RedLed(LedError<<RedLedPin as OutputPin>::Error, <RedLedTimer as Timer<1_000>>::Error>),
    XAxis(AxisError<XAxisDriverError>),
}

pub struct CommandCenter {
    pub actors: CommandCenterActors,
    pub current_actor: Option<CommandActor>,
}

impl CommandCenter {
    pub fn new(resources: CommandCenterResources) -> Self {
        let gpiob = resources.GPIOB.split();
        let gpiog = resources.GPIOG.split();

        let green_led = Led::new(
            gpiob.pb0.into_push_pull_output(),
            resources.TIM3.counter_ms(resources.clocks),
        );
        let blue_led = Led::new(
            gpiob.pb7.into_push_pull_output(),
            resources.TIM4.counter_ms(resources.clocks),
        );
        let red_led = Led::new(
            gpiob.pb14.into_push_pull_output(),
            resources.TIM5.counter_ms(resources.clocks),
        );

        let x_axis = Axis::new_dq542ma(
            gpiog.pg9.into_push_pull_output(),
            gpiog.pg14.into_push_pull_output(),
            EmbeddedTimeCounter(resources.TIM6.counter_us(resources.clocks)),
            0.001_f64,
            1_000_f64,
        );

        Self {
            current_actor: None,
            actors: CommandCenterActors {
                green_led,
                blue_led,
                red_led,
                x_axis,
            },
        }
    }
}

impl ActorReceive for CommandCenter {
    type Message = Command;

    fn receive(&mut self, command: &Self::Message) {
        match command {
            Command::GreenLed(message) => {
                self.actors.green_led.receive(message);
                self.current_actor = Some(CommandActor::GreenLed);
            }

            Command::BlueLed(message) => {
                self.actors.blue_led.receive(message);
                self.current_actor = Some(CommandActor::BlueLed);
            }
            Command::RedLed(message) => {
                self.actors.red_led.receive(message);
                self.current_actor = Some(CommandActor::RedLed);
            }
            Command::XAxis(message) => {
                self.actors.x_axis.receive(message);
                self.current_actor = Some(CommandActor::XAxis);
            }
        }
    }
}

impl ActorPoll for CommandCenter {
    type Error = CommandError;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        match self.current_actor {
            None => Poll::Ready(Ok(())),
            Some(CommandActor::GreenLed) => self
                .actors
                .green_led
                .poll()
                .map_err(|err| CommandError::GreenLed(err)),
            Some(CommandActor::BlueLed) => self
                .actors
                .blue_led
                .poll()
                .map_err(|err| CommandError::BlueLed(err)),
            Some(CommandActor::RedLed) => self
                .actors
                .red_led
                .poll()
                .map_err(|err| CommandError::RedLed(err)),
            Some(CommandActor::XAxis) => self
                .actors
                .x_axis
                .poll()
                .map_err(|err| CommandError::XAxis(err)),
        }
    }
}
