use core::task::Poll;
use stm32f7xx_hal::{
    gpio::{Output, Pin, PushPull},
    pac,
    prelude::*,
    rcc::Clocks,
    timer::{counter::CounterMs, TimerExt},
};

use crate::actor::{ActorPoll, ActorReceive};
use crate::actuators::led::{Led, LedBlinkMessage};
use crate::error::Error;

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

pub struct CommandCenterActors {
    pub green_led: Led<Pin<'B', 0, Output<PushPull>>, CounterMs<pac::TIM3>>,
    pub blue_led: Led<Pin<'B', 7, Output<PushPull>>, CounterMs<pac::TIM4>>,
    pub red_led: Led<Pin<'B', 14, Output<PushPull>>, CounterMs<pac::TIM5>>,
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
    fn poll(&mut self) -> Poll<Result<(), Error>> {
        match self.current_actor {
            None => Poll::Ready(Err(Error::Other)),
            Some(CommandActor::GreenLed) => self.actors.green_led.poll(),
            Some(CommandActor::BlueLed) => self.actors.blue_led.poll(),
            Some(CommandActor::RedLed) => self.actors.red_led.poll(),
        }
    }
}
