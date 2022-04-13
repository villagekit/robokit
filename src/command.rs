// extern crate alloc;

// use alloc::boxed::Box;
use embedded_hal::digital::v2::OutputPin;
use enum_dispatch::enum_dispatch;
use fugit_timer::Timer;
use stm32f7xx_hal::{
    gpio::{Output, Pin, PushPull},
    pac,
    prelude::*,
    rcc::Clocks,
    timer::{counter::CounterMs, TimerExt},
};

use crate::actor::{ActorFuture, ActorReceive};
use crate::actuators::led::{Led, LedBlinkMessage};

pub enum Command {
    GreenLed(LedBlinkMessage),
    BlueLed(LedBlinkMessage),
    RedLed(LedBlinkMessage),
}

#[allow(non_snake_case)]
pub struct CommandActorsResources<'a> {
    pub GPIOB: pac::GPIOB,
    pub TIM3: pac::TIM3,
    pub TIM4: pac::TIM4,
    pub TIM5: pac::TIM5,
    pub clocks: &'a Clocks,
}

#[enum_dispatch(Actor)]
pub enum AnyActor<'a, LedPin, LedTimer>
where
    LedPin: OutputPin,
    LedTimer: Timer<1_000>,
{
    Led(Led<'a, LedPin, LedTimer>),
}

// TODO
// or a new approach, what if this becomes a CommandCenter state machine
// when a new command comes in, this sets the current actor.
// so only that actor is polled.
// and this way, objects don't have to come and go.
// even better, if this itself is an actor!

pub struct CommandActors<'a> {
    pub green_led: AnyActor<'a, Pin<'B', 0, Output<PushPull>>, CounterMs<pac::TIM3>>,
    pub blue_led: AnyActor<'a, Pin<'B', 7, Output<PushPull>>, CounterMs<pac::TIM4>>,
    pub red_led: AnyActor<'a, Pin<'B', 14, Output<PushPull>>, CounterMs<pac::TIM5>>,
}

impl<'a> CommandActors<'a> {
    pub fn new(resources: CommandActorsResources) -> Self {
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
            green_led: AnyActor::Led(green_led),
            blue_led: AnyActor::Led(blue_led),
            red_led: AnyActor::Led(red_led),
        }
    }

    pub fn run(&'a mut self, command: &Command) -> AnyActor< {
        match command {
            Command::GreenLed(message) => {
                self.green_led.receive(command);
                self.green_led
            }

            Command::BlueLed(message) => {
                self.blue_led.receive(command);
                self.blue_led
            }
            Command::RedLed(message) => {
                self.red_led.receive(command);
                self.red_led
            }
        }
    }
}
