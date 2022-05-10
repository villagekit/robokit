use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use fugit_timer::Timer;
use stm32f7xx_hal::{
    gpio::{Input, Output, Pin, PushPull},
    pac,
    prelude::*,
    rcc::{BusTimerClock, Clocks},
    timer::{
        counter::{Counter, CounterUs},
        /*Error as TimerError,*/ TimerExt,
    },
};

use crate::actor::{
    ActorOutbox, ActorOutboxConsumer, ActorOutboxProducer, ActorPoll, ActorPost, ActorReceive,
};
use crate::actuators::axis::{
    Axis, AxisDriverDQ542MA, AxisDriverErrorDQ542MA, AxisError, AxisMoveMessage,
};
use crate::actuators::led::{Led, LedBlinkMessage, LedError};
use crate::sensors::switch::{Switch, SwitchError, SwitchUpdate};

/* actuators */

const LED_TIMER_FREQ: u32 = 1_000_000;
type GreenLedPin = Pin<'B', 0, Output<PushPull>>;
type GreenLedTimer = CounterUs<pac::TIM9>;
type GreenLedError =
    LedError<<GreenLedPin as OutputPin>::Error, <GreenLedTimer as Timer<LED_TIMER_FREQ>>::Error>;
type BlueLedPin = Pin<'B', 7, Output<PushPull>>;
type BlueLedTimer = CounterUs<pac::TIM10>;
type BlueLedError =
    LedError<<BlueLedPin as OutputPin>::Error, <BlueLedTimer as Timer<LED_TIMER_FREQ>>::Error>;
type RedLedPin = Pin<'B', 14, Output<PushPull>>;
type RedLedTimer = CounterUs<pac::TIM11>;
type RedLedError =
    LedError<<RedLedPin as OutputPin>::Error, <RedLedTimer as Timer<LED_TIMER_FREQ>>::Error>;
const X_AXIS_TIMER_FREQ: u32 = 1_000_000;
type XAxisDirPin = Pin<'G', 9, Output<PushPull>>; // D0
type XAxisStepPin = Pin<'G', 14, Output<PushPull>>; // D1
type XAxisTimer = Counter<pac::TIM3, X_AXIS_TIMER_FREQ>;
type XAxisDriver = AxisDriverDQ542MA<XAxisDirPin, XAxisStepPin, XAxisTimer, X_AXIS_TIMER_FREQ>;
type XAxisDriverError =
    AxisDriverErrorDQ542MA<XAxisDirPin, XAxisStepPin, XAxisTimer, X_AXIS_TIMER_FREQ>;
type XAxisError = AxisError<XAxisDriverError>;

#[derive(Format)]
pub enum Command {
    GreenLed(LedBlinkMessage<LED_TIMER_FREQ>),
    BlueLed(LedBlinkMessage<LED_TIMER_FREQ>),
    RedLed(LedBlinkMessage<LED_TIMER_FREQ>),
    XAxis(AxisMoveMessage),
}

pub enum CommandActuator {
    GreenLed,
    BlueLed,
    RedLed,
    XAxis,
}

/* sensors */
type UserButtonPin = Pin<'C', 13, Input>;
type UserButtonError = SwitchError<<UserButtonPin as InputPin>::Error>;

#[allow(non_snake_case)]
pub struct CommandCenterResources<'a> {
    pub GPIOB: pac::GPIOB,
    pub GPIOC: pac::GPIOC,
    pub GPIOG: pac::GPIOG,
    pub TIM3: pac::TIM3,
    pub TIM9: pac::TIM9,
    pub TIM10: pac::TIM10,
    pub TIM11: pac::TIM11,
    pub clocks: &'a Clocks,
}

pub struct CommandCenterActuators {
    pub green_led: Led<GreenLedPin, GreenLedTimer, LED_TIMER_FREQ>,
    pub blue_led: Led<BlueLedPin, BlueLedTimer, LED_TIMER_FREQ>,
    pub red_led: Led<RedLedPin, RedLedTimer, LED_TIMER_FREQ>,
    pub x_axis: Axis<XAxisDriver>,
}

#[derive(Debug)]
pub enum CommandError {
    GreenLed(GreenLedError),
    BlueLed(BlueLedError),
    RedLed(RedLedError),
    XAxis(XAxisError),
}

pub struct CommandCenterSensors<'a> {
    pub user_button: Switch<UserButtonPin, ActorOutboxProducer<'a, SwitchUpdate>>,
}

pub struct CommandCenterSensorOutboxes {
    pub user_button: ActorOutbox<SwitchUpdate>,
}

pub struct CommandCenterSensorConsumers<'a> {
    pub user_button: ActorOutboxConsumer<'a, SwitchUpdate>,
}

pub struct CommandCenter<'a> {
    pub actuators: CommandCenterActuators,
    pub current_actuator: Option<CommandActuator>,
    pub sensors: CommandCenterSensors<'a>,
    // pub sensor_outboxes: CommandCenterSensorOutboxes,
    pub sensor_consumers: CommandCenterSensorConsumers<'a>,
}

impl<'a> CommandCenter<'a> {
    pub fn new(resources: CommandCenterResources) -> Self {
        let gpiob = resources.GPIOB.split();
        let gpioc = resources.GPIOC.split();
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

        let max_acceleration_in_millimeters_per_sec_per_sec = 20_f64;

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

        let mut user_button_outbox = ActorOutbox::<SwitchUpdate>::new();
        let (user_button_outbox_producer, user_button_outbox_consumer) = user_button_outbox.split();
        let user_button = Switch::new(
            gpioc.pc13.into_floating_input(),
            user_button_outbox_producer,
        );

        Self {
            current_actuator: None,
            actuators: CommandCenterActuators {
                green_led,
                blue_led,
                red_led,
                x_axis,
            },
            sensors: CommandCenterSensors { user_button },
            /*
            sensor_outboxes: CommandCenterSensorOutboxes {
                user_button: user_button_outbox,
            },
            */
            sensor_consumers: CommandCenterSensorConsumers {
                user_button: user_button_outbox_consumer,
            },
        }
    }
}

impl<'a> ActorReceive for CommandCenter<'a> {
    type Message = Command;

    fn receive(&mut self, command: &Self::Message) {
        match command {
            Command::GreenLed(message) => {
                self.actuators.green_led.receive(message);
                self.current_actuator = Some(CommandActuator::GreenLed);
            }

            Command::BlueLed(message) => {
                self.actuators.blue_led.receive(message);
                self.current_actuator = Some(CommandActuator::BlueLed);
            }
            Command::RedLed(message) => {
                self.actuators.red_led.receive(message);
                self.current_actuator = Some(CommandActuator::RedLed);
            }
            Command::XAxis(message) => {
                self.actuators.x_axis.receive(message);
                self.current_actuator = Some(CommandActuator::XAxis);
            }
        }
    }
}

impl<'a> ActorPoll for CommandCenter<'a> {
    type Error = CommandError;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        match self.current_actuator {
            None => Poll::Ready(Ok(())),
            Some(CommandActuator::GreenLed) => self
                .actuators
                .green_led
                .poll()
                .map_err(|err| CommandError::GreenLed(err)),
            Some(CommandActuator::BlueLed) => self
                .actuators
                .blue_led
                .poll()
                .map_err(|err| CommandError::BlueLed(err)),
            Some(CommandActuator::RedLed) => self
                .actuators
                .red_led
                .poll()
                .map_err(|err| CommandError::RedLed(err)),
            Some(CommandActuator::XAxis) => self
                .actuators
                .x_axis
                .poll()
                .map_err(|err| CommandError::XAxis(err)),
        }
    }
}
