#![no_main]
#![no_std]

use gridbot_tahi as _;

use core::task::Poll;
use cortex_m_rt::entry;
use defmt::Debug2Format;
use fugit::ExtU32;
use stm32f7xx_hal::{
    gpio::{self, Alternate, Floating, Input, Output, Pin, PullUp, PushPull},
    pac,
    prelude::*,
    rcc::BusTimerClock,
    serial::{Config as SerialConfig, DataBits as SerialDataBits, Parity as SerialParity, Serial},
    timer::Counter,
    watchdog,
};

use robokit::{
    actuators::{
        axis::{AxisDevice, AxisLimitSide},
        led::LedDevice,
        spindle::{SpindleDevice, SpindleDriverJmcHsv57},
    },
    machine::Machine,
    runner::{Runner, RunnerAxes, RunnerLeds, RunnerSpindles},
    sensors::{
        switch::{SwitchActiveHigh, SwitchActiveLow, SwitchDevice, SwitchStatus},
        Sensor,
    },
    timer::{setup as timer_setup, tick as timer_tick, SubTimer, TICK_TIMER_HZ},
};

use gridbot_tahi::init_heap;

pub const TICK_TIMER_MAX: u32 = u32::MAX;
pub type TickTimer = Counter<pac::TIM5, TICK_TIMER_HZ>;

type UserButtonPin = Pin<'C', 13, Input<Floating>>;
type UserButtonTimer = SubTimer;
// type UserButtonError = SwitchError<<UserButtonPin as InputPin>::Error>;
type UserButton = SwitchDevice<UserButtonPin, SwitchActiveHigh, UserButtonTimer, TICK_TIMER_HZ>;

/* actuators */

type GreenLedPin = Pin<'B', 0, Output<PushPull>>;
type GreenLedTimer = SubTimer;
type BlueLedPin = Pin<'B', 7, Output<PushPull>>;
type BlueLedTimer = SubTimer;
type RedLedPin = Pin<'B', 14, Output<PushPull>>;
type RedLedTimer = SubTimer;

const X_AXIS_TIMER_HZ: u32 = 1_000_000;
type XAxisDirPin = Pin<'G', 9, Output<PushPull>>; // D0
type XAxisStepPin = Pin<'G', 14, Output<PushPull>>; // D1
type XAxisTimer = Counter<pac::TIM3, X_AXIS_TIMER_HZ>;

type MainSpindleSerial = Serial<pac::USART2, (gpio::PD5<Alternate<7>>, gpio::PD6<Alternate<7>>)>;
type MainSpindleDriver = SpindleDriverJmcHsv57<MainSpindleSerial>;

/* sensors */
type XAxisLimitMinPin = Pin<'F', 15, Input<PullUp>>; // D2
type XAxisLimitMinTimer = SubTimer;
type XAxisLimitMin =
    SwitchDevice<XAxisLimitMinPin, SwitchActiveLow, XAxisLimitMinTimer, TICK_TIMER_HZ>;
type XAxisLimitMaxPin = Pin<'F', 14, Input<PullUp>>; // D4
type XAxisLimitMaxTimer = SubTimer;
type XAxisLimitMax =
    SwitchDevice<XAxisLimitMaxPin, SwitchActiveLow, XAxisLimitMaxTimer, TICK_TIMER_HZ>;

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
    let gpiof = p.GPIOF.split();
    let gpiog = p.GPIOG.split();

    let mut tick_timer: TickTimer = p.TIM5.counter_us(&clocks);
    timer_setup(&mut tick_timer, TICK_TIMER_MAX).unwrap();

    let user_button_pin: UserButtonPin = gpioc.pc13.into_floating_input();
    let user_button_timer: UserButtonTimer = SubTimer::new();
    let mut user_button: UserButton = SwitchDevice::new(user_button_pin, user_button_timer);

    let green_led_pin: GreenLedPin = gpiob.pb0.into_push_pull_output();
    let green_led_timer: GreenLedTimer = SubTimer::new();
    let green_led = LedDevice::new(green_led_pin, green_led_timer);
    let blue_led_pin: BlueLedPin = gpiob.pb7.into_push_pull_output();
    let blue_led_timer: BlueLedTimer = SubTimer::new();
    let blue_led = LedDevice::new(blue_led_pin, blue_led_timer);
    let red_led_pin: RedLedPin = gpiob.pb14.into_push_pull_output();
    let red_led_timer: RedLedTimer = SubTimer::new();
    let red_led = LedDevice::new(red_led_pin, red_led_timer);

    defmt::println!(
        "Stepper timer clock: {}",
        <pac::TIM3 as BusTimerClock>::timer_clock(&clocks)
    );

    let max_acceleration_in_millimeters_per_sec_per_sec = 20_f64;

    // https://www.makerstore.com.au/product/gear-m1/
    let steps_per_revolution = 6400_f64;
    let millimeters_per_revolution = 125.66_f64;
    let steps_per_millimeter = steps_per_revolution * (1_f64 / millimeters_per_revolution);

    defmt::println!("Steps per mm: {}", steps_per_millimeter);

    let x_axis_dir_pin: XAxisDirPin = gpiog.pg9.into_push_pull_output();
    let x_axis_step_pin: XAxisStepPin = gpiog.pg14.into_push_pull_output();
    let x_axis_timer: XAxisTimer = p.TIM3.counter(&clocks);
    let x_axis_limit_min_pin: XAxisLimitMinPin = gpiof.pf15.into_pull_up_input();
    let x_axis_limit_min_timer: XAxisLimitMinTimer = SubTimer::new();
    let x_axis_limit_min: XAxisLimitMin =
        SwitchDevice::new_active_low(x_axis_limit_min_pin, x_axis_limit_min_timer);
    let x_axis_limit_max_pin: XAxisLimitMaxPin = gpiof.pf14.into_pull_up_input();
    let x_axis_limit_max_timer: XAxisLimitMaxTimer = SubTimer::new();
    let x_axis_limit_max: XAxisLimitMax =
        SwitchDevice::new_active_low(x_axis_limit_max_pin, x_axis_limit_max_timer);
    let x_axis = AxisDevice::new_dq542ma(
        x_axis_dir_pin,
        x_axis_step_pin,
        x_axis_timer,
        max_acceleration_in_millimeters_per_sec_per_sec,
        steps_per_millimeter,
        x_axis_limit_min,
        x_axis_limit_max,
        AxisLimitSide::Min,
    );

    let main_spindle_serial_tx = gpiod.pd5.into_alternate();
    let main_spindle_serial_rx = gpiod.pd6.into_alternate();
    let main_spindle_serial: MainSpindleSerial = Serial::new(
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
    let main_spindle_driver: MainSpindleDriver = SpindleDriverJmcHsv57::new(main_spindle_serial);
    let main_spindle = SpindleDevice::new(main_spindle_driver);

    let runner = Runner::new(
        RunnerLeds {
            green_led,
            blue_led,
            red_led,
        },
        RunnerAxes { x_axis },
        RunnerSpindles { main_spindle },
    );
    let mut machine = Machine::new(runner);

    let mut iwdg = watchdog::IndependentWatchdog::new(p.IWDG);

    iwdg.start(2.millis());

    loop {
        timer_tick(&mut tick_timer, TICK_TIMER_MAX).unwrap();

        if let Some(user_button_update) = user_button.sense().expect("Error reading user button") {
            if let SwitchStatus::On = user_button_update.status {
                machine.toggle();
            }
        }

        if let Poll::Ready(Err(err)) = machine.poll() {
            defmt::println!("Unexpected error: {}", Debug2Format(&err));

            machine.stop();
            loop {
                match machine.poll() {
                    _ => {}
                }
            }
        }

        iwdg.feed();
    }
}
