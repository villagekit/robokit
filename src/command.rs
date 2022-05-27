use core::task::Poll;
use defmt::Format;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use fugit_timer::Timer;
use heapless::Deque;
use stm32f7xx_hal::{
    gpio::{self, Alternate, Floating, Input, Output, Pin, PushPull},
    pac,
    serial::Serial,
    timer::counter::Counter,
};

use crate::actor::{ActorPoll, ActorReceive, ActorSense};
use crate::actuators::axis::{
    Axis, AxisDriverDQ542MA, AxisDriverErrorDQ542MA, AxisError, AxisLimitMessage, AxisLimitSide,
    AxisLimitStatus, AxisMoveMessage,
};
use crate::actuators::led::{Led, LedBlinkMessage, LedError};
use crate::actuators::spindle::{Spindle, SpindleDriverJmcHsv57, SpindleError, SpindleSetMessage};
use crate::sensors::switch::{Switch, SwitchActiveHigh, SwitchError, SwitchStatus};
use crate::timer::{SubTimer, TICK_TIMER_HZ};

/* actuators */

type GreenLedPin = Pin<'B', 0, Output<PushPull>>;
type GreenLedTimer = SubTimer;
type GreenLedError =
    LedError<<GreenLedPin as OutputPin>::Error, <GreenLedTimer as Timer<TICK_TIMER_HZ>>::Error>;
type BlueLedPin = Pin<'B', 7, Output<PushPull>>;
type BlueLedTimer = SubTimer;
type BlueLedError =
    LedError<<BlueLedPin as OutputPin>::Error, <BlueLedTimer as Timer<TICK_TIMER_HZ>>::Error>;
type RedLedPin = Pin<'B', 14, Output<PushPull>>;
type RedLedTimer = SubTimer;
type RedLedError =
    LedError<<RedLedPin as OutputPin>::Error, <RedLedTimer as Timer<TICK_TIMER_HZ>>::Error>;

const X_AXIS_TIMER_HZ: u32 = 1_000_000;
type XAxisDirPin = Pin<'G', 9, Output<PushPull>>; // D0
type XAxisStepPin = Pin<'G', 14, Output<PushPull>>; // D1
type XAxisTimer = Counter<pac::TIM3, X_AXIS_TIMER_HZ>;
type XAxisDriver = AxisDriverDQ542MA<XAxisDirPin, XAxisStepPin, XAxisTimer, X_AXIS_TIMER_HZ>;
type XAxisDriverError =
    AxisDriverErrorDQ542MA<XAxisDirPin, XAxisStepPin, XAxisTimer, X_AXIS_TIMER_HZ>;
type XAxisError = AxisError<XAxisDriverError>;

type MainSpindleSerial = Serial<pac::USART2, (gpio::PD5<Alternate<7>>, gpio::PD6<Alternate<7>>)>;
type MainSpindleDriver = SpindleDriverJmcHsv57<MainSpindleSerial>;
type MainSpindleError = SpindleError<MainSpindleDriver>;

#[derive(Clone, Copy, Debug, Format)]
pub enum Command {
    GreenLed(LedBlinkMessage<TICK_TIMER_HZ>),
    BlueLed(LedBlinkMessage<TICK_TIMER_HZ>),
    RedLed(LedBlinkMessage<TICK_TIMER_HZ>),
    XAxis(AxisMoveMessage),
    MainSpindle(SpindleSetMessage),
}

/* sensors */
type XAxisLimitMinPin = Pin<'F', 15, Input<Floating>>; // D2
type XAxisLimitMinTimer = SubTimer;
type XAxisLimitMinError = SwitchError<
    <XAxisLimitMinPin as InputPin>::Error,
    <XAxisLimitMinTimer as Timer<TICK_TIMER_HZ>>::Error,
>;
type XAxisLimitMin = Switch<XAxisLimitMinPin, SwitchActiveHigh, XAxisLimitMinTimer, TICK_TIMER_HZ>;
type XAxisLimitMaxPin = Pin<'E', 13, Input<Floating>>; // D3
type XAxisLimitMaxTimer = SubTimer;
type XAxisLimitMaxError = SwitchError<
    <XAxisLimitMaxPin as InputPin>::Error,
    <XAxisLimitMaxTimer as Timer<TICK_TIMER_HZ>>::Error,
>;
type XAxisLimitMax = Switch<XAxisLimitMaxPin, SwitchActiveHigh, XAxisLimitMaxTimer, TICK_TIMER_HZ>;

pub struct CommandCenterResources {
    pub green_led_pin: GreenLedPin,
    pub green_led_timer: GreenLedTimer,
    pub blue_led_pin: BlueLedPin,
    pub blue_led_timer: BlueLedTimer,
    pub red_led_pin: RedLedPin,
    pub red_led_timer: RedLedTimer,
    pub x_axis_dir_pin: XAxisDirPin,
    pub x_axis_step_pin: XAxisStepPin,
    pub x_axis_timer: XAxisTimer,
    pub x_axis_limit_min_pin: XAxisLimitMinPin,
    pub x_axis_limit_min_timer: XAxisLimitMinTimer,
    pub x_axis_limit_max_pin: XAxisLimitMaxPin,
    pub x_axis_limit_max_timer: XAxisLimitMaxTimer,
    pub main_spindle_serial: MainSpindleSerial,
}

pub struct CommandCenterActuators {
    pub green_led: Led<GreenLedPin, GreenLedTimer, TICK_TIMER_HZ>,
    pub blue_led: Led<BlueLedPin, BlueLedTimer, TICK_TIMER_HZ>,
    pub red_led: Led<RedLedPin, RedLedTimer, TICK_TIMER_HZ>,
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
    pub x_axis_limit_min: XAxisLimitMin,
    pub x_axis_limit_max: XAxisLimitMax,
}

#[derive(Debug)]
pub enum SensorError {
    XAxisLimitMin(XAxisLimitMinError),
    XAxisLimitMax(XAxisLimitMaxError),
}

pub struct CommandCenter {
    pub active_commands: Deque<Command, 8>,
    pub actuators: CommandCenterActuators,
    pub sensors: CommandCenterSensors,
}

impl CommandCenter {
    pub fn new(res: CommandCenterResources) -> Self {
        let green_led = Led::new(res.green_led_pin, res.green_led_timer);
        let blue_led = Led::new(res.blue_led_pin, res.blue_led_timer);
        let red_led = Led::new(res.red_led_pin, res.red_led_timer);

        let max_acceleration_in_millimeters_per_sec_per_sec = 20_f64;

        let steps_per_revolution = 6400_f64;
        let leadscrew_starts = 4_f64;
        let leadscrew_pitch = 2_f64;
        let millimeters_per_revolution = leadscrew_starts * leadscrew_pitch;
        let steps_per_millimeter = steps_per_revolution / millimeters_per_revolution;

        defmt::println!("Steps per mm: {}", steps_per_millimeter);

        let x_axis = Axis::new_dq542ma(
            res.x_axis_dir_pin,
            res.x_axis_step_pin,
            res.x_axis_timer,
            max_acceleration_in_millimeters_per_sec_per_sec,
            steps_per_millimeter,
        );
        let x_axis_limit_min = Switch::new(res.x_axis_limit_min_pin, res.x_axis_limit_min_timer);
        let x_axis_limit_max = Switch::new(res.x_axis_limit_max_pin, res.x_axis_limit_max_timer);

        let main_spindle_driver = SpindleDriverJmcHsv57::new(res.main_spindle_serial);
        let main_spindle = Spindle::new(main_spindle_driver);

        Self {
            active_commands: Deque::new(),
            actuators: CommandCenterActuators {
                green_led,
                blue_led,
                red_led,
                x_axis,
                main_spindle,
            },
            sensors: CommandCenterSensors {
                x_axis_limit_min,
                x_axis_limit_max,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct ResetMessage {}

impl ActorReceive<ResetMessage> for CommandCenter {
    fn receive(&mut self, _message: &ResetMessage) {
        self.active_commands.clear()
    }
}

impl ActorReceive<Command> for CommandCenter {
    fn receive(&mut self, command: &Command) {
        match command {
            Command::GreenLed(message) => {
                self.actuators.green_led.receive(message);
            }
            Command::BlueLed(message) => {
                self.actuators.blue_led.receive(message);
            }
            Command::RedLed(message) => {
                self.actuators.red_led.receive(message);
            }
            Command::XAxis(message) => {
                self.actuators.x_axis.receive(message);
            }
            Command::MainSpindle(message) => {
                self.actuators.main_spindle.receive(message);
            }
        }

        self.active_commands.push_back(*command).unwrap();
    }
}

impl CommandCenter {
    pub fn sense(&mut self) -> Result<(), SensorError> {
        let axis_limit_min_message = AxisLimitMessage {
            side: AxisLimitSide::Min,
            status: AxisLimitStatus::Under,
        };
        self.actuators.x_axis.receive(&axis_limit_min_message);
        /*
        if let Some(axis_limit_update) = self
            .sensors
            .x_axis_limit_min
            .sense()
            .map_err(|err| SensorError::XAxisLimitMin(err))?
        {
            let axis_limit_status = match axis_limit_update.status {
                SwitchStatus::On => AxisLimitStatus::Over,
                SwitchStatus::Off => AxisLimitStatus::Under,
            };
            let axis_limit_min_message = AxisLimitMessage {
                side: AxisLimitSide::Min,
                status: axis_limit_status,
            };
            self.actuators.x_axis.receive(&axis_limit_min_message);
        }
        */

        let axis_limit_max_message = AxisLimitMessage {
            side: AxisLimitSide::Max,
            status: AxisLimitStatus::Under,
        };
        self.actuators.x_axis.receive(&axis_limit_max_message);
        /*
        if let Some(axis_limit_update) = self
            .sensors
            .x_axis_limit_max
            .sense()
            .map_err(|err| SensorError::XAxisLimitMax(err))?
        {
            let axis_limit_status = match axis_limit_update.status {
                SwitchStatus::On => AxisLimitStatus::Over,
                SwitchStatus::Off => AxisLimitStatus::Under,
            };
            let axis_limit_max_message = AxisLimitMessage {
                side: AxisLimitSide::Max,
                status: axis_limit_status,
            };
            self.actuators.x_axis.receive(&axis_limit_max_message);
        }
        */

        Ok(())
    }
}

impl ActorPoll for CommandCenter {
    type Error = ActuatorError;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        let num_commands = self.active_commands.len();
        for _command_index in 0..num_commands {
            let command = self.active_commands.pop_front().unwrap();
            let result = match command {
                Command::GreenLed(_) => self
                    .actuators
                    .green_led
                    .poll()
                    .map_err(|err| ActuatorError::GreenLed(err)),
                Command::BlueLed(_) => self
                    .actuators
                    .blue_led
                    .poll()
                    .map_err(|err| ActuatorError::BlueLed(err)),
                Command::RedLed(_) => self
                    .actuators
                    .red_led
                    .poll()
                    .map_err(|err| ActuatorError::RedLed(err)),
                Command::XAxis(_) => self
                    .actuators
                    .x_axis
                    .poll()
                    .map_err(|err| ActuatorError::XAxis(err)),
                Command::MainSpindle(_) => self
                    .actuators
                    .main_spindle
                    .poll()
                    .map_err(|err| ActuatorError::MainSpindle(err)),
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
