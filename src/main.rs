#![no_main]
#![no_std]

use gridbot as _;

use core::task::Poll;
use cortex_m_rt::entry;
use defmt::Debug2Format;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use fugit::ExtU32;
use fugit_timer::Timer;
use stm32f7xx_hal::{
    gpio::{self, Alternate, Floating, Input, Output, Pin, PushPull},
    pac,
    prelude::*,
    rcc::BusTimerClock,
    serial::{Config as SerialConfig, DataBits as SerialDataBits, Parity as SerialParity, Serial},
    timer::Counter,
    watchdog,
};

use gridbot::{
    actor::{ActorPoll, ActorReceive, ActorSense},
    actuators::axis::{
        AxisDevice, AxisDriverDQ542MA, AxisDriverErrorDQ542MA, AxisError, AxisLimitMessage,
        AxisLimitSide, AxisLimitStatus, AxisMoveMessage,
    },
    actuators::led::{LedDevice, LedError},
    actuators::spindle::{
        SpindleDevice, SpindleDriver, SpindleDriverJmcHsv57, SpindleError, SpindleSetMessage,
    },
    command::{CommandCenter, CommandCenterResources},
    init_heap,
    machine::{Machine, StopMessage, ToggleMessage},
    sensors::switch::{SwitchActiveHigh, SwitchDevice, SwitchError, SwitchStatus},
    timer::{setup as timer_setup, tick as timer_tick, SubTimer, TICK_TIMER_HZ},
};

pub const TICK_TIMER_MAX: u32 = u32::MAX;
pub type TickTimer = Counter<pac::TIM5, TICK_TIMER_HZ>;

type UserButtonPin = Pin<'C', 13, Input<Floating>>;
type UserButtonTimer = SubTimer;
// type UserButtonError = SwitchError<<UserButtonPin as InputPin>::Error>;
type UserButton = SwitchDevice<UserButtonPin, SwitchActiveHigh, UserButtonTimer, TICK_TIMER_HZ>;

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

/* sensors */
type XAxisLimitMinPin = Pin<'F', 15, Input<Floating>>; // D2
type XAxisLimitMinTimer = SubTimer;
type XAxisLimitMinError = SwitchError<
    <XAxisLimitMinPin as InputPin>::Error,
    <XAxisLimitMinTimer as Timer<TICK_TIMER_HZ>>::Error,
>;
type XAxisLimitMin =
    SwitchDevice<XAxisLimitMinPin, SwitchActiveHigh, XAxisLimitMinTimer, TICK_TIMER_HZ>;
type XAxisLimitMaxPin = Pin<'E', 13, Input<Floating>>; // D3
type XAxisLimitMaxTimer = SubTimer;
type XAxisLimitMaxError = SwitchError<
    <XAxisLimitMaxPin as InputPin>::Error,
    <XAxisLimitMaxTimer as Timer<TICK_TIMER_HZ>>::Error,
>;
type XAxisLimitMax =
    SwitchDevice<XAxisLimitMaxPin, SwitchActiveHigh, XAxisLimitMaxTimer, TICK_TIMER_HZ>;

#[entry]
fn main() -> ! {
    init_heap();

    defmt::println!("Init!");

    let p = pac::Peripherals::take().unwrap();

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();
    let gpiod = p.GPIOD.split();
    let gpioe = p.GPIOE.split();
    let gpiof = p.GPIOF.split();
    let gpiog = p.GPIOG.split();

    let mut tick_timer: TickTimer = p.TIM5.counter_us(&clocks);
    timer_setup(&mut tick_timer, TICK_TIMER_MAX).unwrap();

    let user_button_pin = gpioc.pc13.into_floating_input();
    let user_button_timer = SubTimer::new();
    let mut user_button: UserButton = SwitchDevice::new(user_button_pin, user_button_timer);

    let green_led_pin = gpiob.pb0.into_push_pull_output();
    let green_led_timer = SubTimer::new();
    let green_led = LedDevice::new(green_led_pin, green_led_timer);
    let blue_led_pin = gpiob.pb7.into_push_pull_output();
    let blue_led_timer = SubTimer::new();
    let blue_led = LedDevice::new(blue_led_pin, blue_led_timer);
    let red_led_pin = gpiob.pb14.into_push_pull_output();
    let red_led_timer = SubTimer::new();
    let red_led = LedDevice::new(red_led_pin, red_led_timer);

    defmt::println!(
        "Stepper timer clock: {}",
        <pac::TIM3 as BusTimerClock>::timer_clock(&clocks)
    );

    let max_acceleration_in_millimeters_per_sec_per_sec = 20_f64;

    let steps_per_revolution = 6400_f64;
    let leadscrew_starts = 4_f64;
    let leadscrew_pitch = 2_f64;
    let millimeters_per_revolution = leadscrew_starts * leadscrew_pitch;
    let steps_per_millimeter = steps_per_revolution / millimeters_per_revolution;

    defmt::println!("Steps per mm: {}", steps_per_millimeter);

    let x_axis_dir_pin = gpiog.pg9.into_push_pull_output();
    let x_axis_step_pin = gpiog.pg14.into_push_pull_output();
    let x_axis_timer = p.TIM3.counter(&clocks);
    let x_axis = AxisDevice::new_dq542ma(
        x_axis_dir_pin,
        x_axis_step_pin,
        x_axis_timer,
        max_acceleration_in_millimeters_per_sec_per_sec,
        steps_per_millimeter,
    );
    let x_axis_limit_min_pin = gpiof.pf15.into_floating_input();
    let x_axis_limit_min_timer = SubTimer::new();
    let x_axis_limit_min: XAxisLimitMin =
        SwitchDevice::new(x_axis_limit_min_pin, x_axis_limit_min_timer);
    let x_axis_limit_max_pin = gpioe.pe13.into_floating_input();
    let x_axis_limit_max_timer = SubTimer::new();
    let x_axis_limit_max: XAxisLimitMax =
        SwitchDevice::new(x_axis_limit_max_pin, x_axis_limit_max_timer);

    let main_spindle_serial_tx = gpiod.pd5.into_alternate();
    let main_spindle_serial_rx = gpiod.pd6.into_alternate();
    let main_spindle_serial = Serial::new(
        p.USART2,
        (main_spindle_serial_tx, main_spindle_serial_rx),
        &clocks,
        SerialConfig {
            baud_rate: 57600.bps(),
            data_bits: SerialDataBits::Bits9,
            parity: SerialParity::ParityEven,
            ..Default::default()
        },
    );

    let command_center = CommandCenter::new(CommandCenterResources {
        green_led_pin,
        green_led_timer,
        blue_led_pin,
        blue_led_timer,
        red_led_pin,
        red_led_timer,
        x_axis_dir_pin,
        x_axis_step_pin,
        x_axis_timer,
        x_axis_limit_min_pin,
        x_axis_limit_min_timer,
        x_axis_limit_max_pin,
        x_axis_limit_max_timer,
        main_spindle_serial,
    });
    let mut machine = Machine::new(command_center);

    let mut iwdg = watchdog::IndependentWatchdog::new(p.IWDG);

    iwdg.start(2.millis());

    loop {
        timer_tick(&mut tick_timer, TICK_TIMER_MAX).unwrap();

        if let Some(user_button_update) = user_button.sense().expect("Error reading user button") {
            if let SwitchStatus::On = user_button_update.status {
                machine.receive(&ToggleMessage {});
            }
        }

        if let Poll::Ready(Err(err)) = machine.poll() {
            defmt::println!("Unexpected error: {}", Debug2Format(&err));

            machine.receive(&StopMessage {});
            loop {
                match machine.poll() {
                    _ => {}
                }
            }
        }

        iwdg.feed();
    }
}
