use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use fugit_timer::Timer;
use stm32f7xx_hal::{
    gpio::{self, Alternate, Floating, Input, Output, Pin, PushPull},
    pac,
    prelude::*,
    rcc::{BusTimerClock, Clocks},
    serial::{
        Config as SerialConfig, Oversampling as SerialOversampling, Parity as SerialParity, Serial,
    },
    timer::{
        counter::{Counter, CounterUs},
        TimerExt,
    },
};

use crate::actor::{ActorPoll, ActorReceive, ActorSense};
use crate::actuators::axis::{
    Axis, AxisDriverDQ542MA, AxisDriverErrorDQ542MA, AxisError, AxisLimitMessage, AxisLimitSide,
    AxisLimitStatus, AxisMoveMessage,
};
use crate::actuators::led::{Led, LedBlinkMessage, LedError};
use crate::actuators::spindle::{Spindle, SpindleDriverJmcHsv57, SpindleError, SpindleSetMessage};
use crate::sensors::switch::{Switch, SwitchActiveHigh, SwitchError, SwitchStatus};

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

type MainSpindleSerial = Serial<pac::USART2, (gpio::PD5<Alternate<7>>, gpio::PD6<Alternate<7>>)>;
type MainSpindleDriver = SpindleDriverJmcHsv57<MainSpindleSerial>;
type MainSpindleError = SpindleError<MainSpindleDriver>;

#[derive(Clone, Copy, Debug, Format)]
pub enum Command {
    GreenLed(LedBlinkMessage<LED_TIMER_FREQ>),
    BlueLed(LedBlinkMessage<LED_TIMER_FREQ>),
    RedLed(LedBlinkMessage<LED_TIMER_FREQ>),
    XAxis(AxisMoveMessage),
    MainSpindle(SpindleSetMessage),
}

#[derive(Clone, Copy, Debug, Format)]
pub enum CommandActuator {
    GreenLed,
    BlueLed,
    RedLed,
    XAxis,
    MainSpindle,
}

/* sensors */
type UserButtonPin = Pin<'C', 13, Input<Floating>>;
type UserButtonError = SwitchError<<UserButtonPin as InputPin>::Error>;

#[allow(non_snake_case)]
pub struct CommandCenterResources<'a> {
    pub GPIOB: pac::GPIOB,
    pub GPIOC: pac::GPIOC,
    pub GPIOD: pac::GPIOD,
    pub GPIOG: pac::GPIOG,
    pub TIM3: pac::TIM3,
    pub TIM9: pac::TIM9,
    pub TIM10: pac::TIM10,
    pub TIM11: pac::TIM11,
    pub USART2: pac::USART2,
    pub clocks: &'a Clocks,
}

pub struct CommandCenterActuators {
    pub green_led: Led<GreenLedPin, GreenLedTimer, LED_TIMER_FREQ>,
    pub blue_led: Led<BlueLedPin, BlueLedTimer, LED_TIMER_FREQ>,
    pub red_led: Led<RedLedPin, RedLedTimer, LED_TIMER_FREQ>,
    pub x_axis: Axis<XAxisDriver>,
    pub main_spindle: Spindle<MainSpindleDriver>,
}

#[derive(Debug)]
pub enum ActuatorError {
    GreenLed(GreenLedError),
    BlueLed(BlueLedError),
    RedLed(RedLedError),
    XAxis(XAxisError),
    MainSpindle(MainSpindleError),
}

pub struct CommandCenterSensors {
    pub user_button: Switch<UserButtonPin, SwitchActiveHigh>,
}

#[derive(Debug)]
pub enum SensorError {
    UserButton(UserButtonError),
}

#[derive(Debug)]
pub enum CommandCenterError {
    Actuator(ActuatorError),
    Sensor(SensorError),
}

pub struct CommandCenter {
    pub actuators: CommandCenterActuators,
    pub current_actuator: Option<CommandActuator>,
    pub sensors: CommandCenterSensors,
}

impl CommandCenter {
    pub fn new(resources: CommandCenterResources) -> Self {
        let gpiob = resources.GPIOB.split();
        let gpioc = resources.GPIOC.split();
        let gpiod = resources.GPIOD.split();
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

        let tx = gpiod.pd5.into_alternate();
        let rx = gpiod.pd6.into_alternate();
        let main_spindle_serial = Serial::new(
            resources.USART2,
            (tx, rx),
            &resources.clocks,
            SerialConfig {
                baud_rate: 57600.bps(),
                oversampling: SerialOversampling::By16,
                character_match: None,
                sysclock: false,
                parity: SerialParity::ParityEven,
            },
        );
        let main_spindle_driver = SpindleDriverJmcHsv57::new(main_spindle_serial);
        let main_spindle = Spindle::new(main_spindle_driver);

        let user_button = Switch::new(gpioc.pc13.into_floating_input());

        Self {
            current_actuator: None,
            actuators: CommandCenterActuators {
                green_led,
                blue_led,
                red_led,
                x_axis,
                main_spindle,
            },
            sensors: CommandCenterSensors { user_button },
        }
    }
}

impl ActorReceive<Command> for CommandCenter {
    fn receive(&mut self, command: &Command) {
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
            Command::MainSpindle(message) => {
                self.actuators.main_spindle.receive(message);
                self.current_actuator = Some(CommandActuator::MainSpindle);
            }
        }
    }
}

impl CommandCenter {
    pub fn update(&mut self) -> Result<(), SensorError> {
        let axis_limit_min_message = AxisLimitMessage {
            side: AxisLimitSide::Min,
            status: AxisLimitStatus::Under,
        };
        self.actuators.x_axis.receive(&axis_limit_min_message);

        if let Some(user_button_update) = self
            .sensors
            .user_button
            .sense()
            .map_err(|err| SensorError::UserButton(err))?
        {
            let axis_limit_status = match user_button_update.status {
                SwitchStatus::On => AxisLimitStatus::Over,
                SwitchStatus::Off => AxisLimitStatus::Under,
            };

            let axis_limit_max_message = AxisLimitMessage {
                side: AxisLimitSide::Max,
                status: axis_limit_status,
            };
            self.actuators.x_axis.receive(&axis_limit_max_message);
        }

        Ok(())
    }
}

impl ActorPoll for CommandCenter {
    type Error = ActuatorError;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        match self.current_actuator {
            None => Poll::Ready(Ok(())),
            Some(CommandActuator::GreenLed) => self
                .actuators
                .green_led
                .poll()
                .map_err(|err| ActuatorError::GreenLed(err)),
            Some(CommandActuator::BlueLed) => self
                .actuators
                .blue_led
                .poll()
                .map_err(|err| ActuatorError::BlueLed(err)),
            Some(CommandActuator::RedLed) => self
                .actuators
                .red_led
                .poll()
                .map_err(|err| ActuatorError::RedLed(err)),
            Some(CommandActuator::XAxis) => self
                .actuators
                .x_axis
                .poll()
                .map_err(|err| ActuatorError::XAxis(err)),
            Some(CommandActuator::MainSpindle) => self
                .actuators
                .main_spindle
                .poll()
                .map_err(|err| ActuatorError::MainSpindle(err)),
        }
    }
}
