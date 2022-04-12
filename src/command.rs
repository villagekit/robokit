extern crate alloc;

use alloc::boxed::Box;
use stm32f7xx_hal::{
    gpio::{Output, Pin, PushPull},
    pac,
    prelude::*,
    rcc::Clocks,
    timer::{counter::CounterMs, TimerExt},
};

use crate::actuator::{Actuator, Future};
use crate::actuators::led::{Led, LedBlinkMessage};

pub enum Command {
    GreenLed(LedBlinkMessage),
    BlueLed(LedBlinkMessage),
    RedLed(LedBlinkMessage),
}

#[allow(non_snake_case)]
pub struct CommandActuatorsResources<'a> {
    pub GPIOB: pac::GPIOB,
    pub TIM3: pac::TIM3,
    pub TIM4: pac::TIM4,
    pub TIM5: pac::TIM5,
    pub clocks: &'a Clocks,
}

pub struct CommandActuators {
    pub green_led: Led<Pin<'B', 0, Output<PushPull>>, CounterMs<pac::TIM3>>,
    pub blue_led: Led<Pin<'B', 7, Output<PushPull>>, CounterMs<pac::TIM4>>,
    pub red_led: Led<Pin<'B', 14, Output<PushPull>>, CounterMs<pac::TIM5>>,
}

impl<'a> CommandActuators {
    pub fn new(resources: CommandActuatorsResources) -> Self {
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
            green_led,
            blue_led,
            red_led,
        }
    }

    pub fn run(&'a mut self, command: &Command) -> Box<dyn Future + 'a> {
        match command {
            Command::GreenLed(message) => Box::new(self.green_led.command(message)),
            Command::BlueLed(message) => Box::new(self.blue_led.command(message)),
            Command::RedLed(message) => Box::new(self.red_led.command(message)),
        }
    }
}
