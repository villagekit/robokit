use core::task::Poll;
use embedded_hal::digital::v2::OutputPin;
use fugit_timer::Timer;
use stm32f7xx_hal::{
    gpio::{Output, Pin, PushPull},
    pac,
    prelude::*,
    rcc::Clocks,
    timer::{counter::CounterMs, /*Error as TimerError,*/ TimerExt},
};

use crate::actor::{ActorPoll, ActorReceive};
use crate::actuators::led::{Led, LedBlinkMessage, LedError};

pub enum Command {
    GreenLed(LedBlinkMessage),
    BlueLed(LedBlinkMessage),
    RedLed(LedBlinkMessage),
}

pub enum CommandActor {
    GreenLed,
    BlueLed,
    RedLed,
}

#[allow(non_snake_case)]
pub struct CommandCenterResources<'a> {
    pub GPIOB: pac::GPIOB,
    pub TIM3: pac::TIM3,
    pub TIM4: pac::TIM4,
    pub TIM5: pac::TIM5,
    pub clocks: &'a Clocks,
}

type GreenLedPin = Pin<'B', 0, Output<PushPull>>;
type GreenLedTimer = CounterMs<pac::TIM3>;
type BlueLedPin = Pin<'B', 7, Output<PushPull>>;
type BlueLedTimer = CounterMs<pac::TIM4>;
type RedLedPin = Pin<'B', 14, Output<PushPull>>;
type RedLedTimer = CounterMs<pac::TIM5>;

pub struct CommandCenterActors {
    pub green_led: Led<GreenLedPin, GreenLedTimer>,
    pub blue_led: Led<BlueLedPin, BlueLedTimer>,
    pub red_led: Led<RedLedPin, RedLedTimer>,
}

#[derive(Debug)]
pub enum CommandError {
    GreenLed(LedError<<GreenLedPin as OutputPin>::Error, <GreenLedTimer as Timer<1_000>>::Error>),
    BlueLed(LedError<<BlueLedPin as OutputPin>::Error, <BlueLedTimer as Timer<1_000>>::Error>),
    RedLed(LedError<<RedLedPin as OutputPin>::Error, <RedLedTimer as Timer<1_000>>::Error>),
}

pub struct CommandCenter {
    pub actors: CommandCenterActors,
    pub current_actor: Option<CommandActor>,
}

impl CommandCenter {
    pub fn new(resources: CommandCenterResources) -> Self {
        let gpiob = resources.GPIOB.split();

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

        Self {
            current_actor: None,
            actors: CommandCenterActors {
                green_led,
                blue_led,
                red_led,
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
        }
    }
}
