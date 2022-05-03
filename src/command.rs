use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use fugit_timer::Timer;
use stm32f7xx_hal::{
    gpio::{Output, Pin, PushPull},
    pac,
    prelude::*,
    rcc::{BusTimerClock, Clocks},
    timer::{
        counter::{Counter, CounterUs},
        /*Error as TimerError,*/ TimerExt,
    },
};

use crate::actor::{ActorPoll, ActorReceive};
use crate::actuators::axis::{
    Axis, AxisDriverDQ542MA, AxisDriverErrorDQ542MA, AxisError, AxisMoveMessage,
};
use crate::actuators::led::{Led, LedBlinkMessage, LedError};

const LED_TIMER_FREQ: u32 = 1_000_000;
type GreenLedPin = Pin<'B', 0, Output<PushPull>>;
type GreenLedTimer = CounterUs<pac::TIM9>;
type BlueLedPin = Pin<'B', 7, Output<PushPull>>;
type BlueLedTimer = CounterUs<pac::TIM10>;
type RedLedPin = Pin<'B', 14, Output<PushPull>>;
type RedLedTimer = CounterUs<pac::TIM11>;
const X_AXIS_TIMER_FREQ: u32 = 1_000_000;
type XAxisDirPin = Pin<'G', 9, Output<PushPull>>; // D0
type XAxisStepPin = Pin<'G', 14, Output<PushPull>>; // D1
type XAxisTimer = Counter<pac::TIM3, X_AXIS_TIMER_FREQ>;
type XAxisDriver = AxisDriverDQ542MA<XAxisDirPin, XAxisStepPin, XAxisTimer, X_AXIS_TIMER_FREQ>;
type XAxisDriverError =
    AxisDriverErrorDQ542MA<XAxisDirPin, XAxisStepPin, XAxisTimer, X_AXIS_TIMER_FREQ>;

#[derive(Format)]
pub enum Command {
    GreenLed(LedBlinkMessage<LED_TIMER_FREQ>),
    BlueLed(LedBlinkMessage<LED_TIMER_FREQ>),
    RedLed(LedBlinkMessage<LED_TIMER_FREQ>),
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
    pub TIM9: pac::TIM9,
    pub TIM10: pac::TIM10,
    pub TIM11: pac::TIM11,
    pub clocks: &'a Clocks,
}

pub struct CommandCenterActors {
    pub green_led: Led<GreenLedPin, GreenLedTimer, LED_TIMER_FREQ>,
    pub blue_led: Led<BlueLedPin, BlueLedTimer, LED_TIMER_FREQ>,
    pub red_led: Led<RedLedPin, RedLedTimer, LED_TIMER_FREQ>,
    pub x_axis: Axis<XAxisDriver>,
}

#[derive(Debug)]
pub enum CommandError {
    GreenLed(
        LedError<
            <GreenLedPin as OutputPin>::Error,
            <GreenLedTimer as Timer<LED_TIMER_FREQ>>::Error,
        >,
    ),
    BlueLed(
        LedError<<BlueLedPin as OutputPin>::Error, <BlueLedTimer as Timer<LED_TIMER_FREQ>>::Error>,
    ),
    RedLed(
        LedError<<RedLedPin as OutputPin>::Error, <RedLedTimer as Timer<LED_TIMER_FREQ>>::Error>,
    ),
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
            resources.TIM9.counter_us(resources.clocks),
        );
        let blue_led = Led::new(
            gpiob.pb7.into_push_pull_output(),
            resources.TIM10.counter_us(resources.clocks),
        );
        let red_led = Led::new(
            gpiob.pb14.into_push_pull_output(),
            resources.TIM11.counter_us(resources.clocks),
        );

        defmt::println!(
            "Stepper timer clock: {}",
            <pac::TIM3 as BusTimerClock>::timer_clock(resources.clocks)
        );

        let max_acceleration_in_millimeters_per_sec_per_sec = 10_f64;

        let steps_per_revolution = 6400_f64;
        let leadscrew_starts = 4_f64;
        let leadscrew_pitch = 2_f64;
        let millimeters_per_revolution = leadscrew_starts * leadscrew_pitch;
        let steps_per_millimeter = steps_per_revolution / millimeters_per_revolution;

        defmt::println!("Steps per mm: {}", steps_per_millimeter);

        let x_axis = Axis::new_dq542ma(
            gpiog.pg9.into_push_pull_output(),
            gpiog.pg14.into_push_pull_output(),
            resources.TIM3.counter(resources.clocks),
            max_acceleration_in_millimeters_per_sec_per_sec,
            steps_per_millimeter,
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
