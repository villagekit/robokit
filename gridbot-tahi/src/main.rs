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
    AxisDevice, AxisLimitSide, LedDevice, RobotBuilder, Sensor, SpindleDevice,
    SpindleDriverJmcHsv57, SubTimer, SuperTimer, SwitchActiveHigh, SwitchActiveLow, SwitchDevice,
    SwitchStatus,
};

use gridbot_tahi::{
    actuators::{AxisSet, LedSet, SpindleSet},
    commands::{get_run_commands, get_start_commands, get_stop_commands},
    init_heap,
};

const ACTIVE_COMMANDS_COUNT: usize = 8;

const TICK_TIMER_MAX: u32 = u32::MAX;
const TICK_TIMER_HZ: u32 = 1_000_000;
type TickTimerDevice = Counter<pac::TIM5, TICK_TIMER_HZ>;
type TickTimer = SuperTimer<TickTimerDevice, TICK_TIMER_HZ>;

type UserButtonPin = Pin<'C', 13, Input<Floating>>;
type UserButtonTimer = SubTimer<TICK_TIMER_HZ>;
type UserButton = SwitchDevice<UserButtonPin, SwitchActiveHigh, UserButtonTimer, TICK_TIMER_HZ>;

/* leds */

type GreenLedPin = Pin<'B', 0, Output<PushPull>>;
type GreenLedTimer = SubTimer<TICK_TIMER_HZ>;
type BlueLedPin = Pin<'B', 7, Output<PushPull>>;
type BlueLedTimer = SubTimer<TICK_TIMER_HZ>;
type RedLedPin = Pin<'B', 14, Output<PushPull>>;
type RedLedTimer = SubTimer<TICK_TIMER_HZ>;

/* limit switches */
type LengthAxisLimitMinPin = Pin<'G', 9, Input<PullUp>>; // D0
type LengthAxisLimitMinTimer = SubTimer<TICK_TIMER_HZ>;
type LengthAxisLimitMin =
    SwitchDevice<LengthAxisLimitMinPin, SwitchActiveLow, LengthAxisLimitMinTimer, TICK_TIMER_HZ>;
type LengthAxisLimitMaxPin = Pin<'G', 14, Input<PullUp>>; // D1
type LengthAxisLimitMaxTimer = SubTimer<TICK_TIMER_HZ>;
type LengthAxisLimitMax =
    SwitchDevice<LengthAxisLimitMaxPin, SwitchActiveLow, LengthAxisLimitMaxTimer, TICK_TIMER_HZ>;

type WidthAxisLimitMinPin = Pin<'F', 15, Input<PullUp>>; // D2
type WidthAxisLimitMinTimer = SubTimer<TICK_TIMER_HZ>;
type WidthAxisLimitMin =
    SwitchDevice<WidthAxisLimitMinPin, SwitchActiveLow, WidthAxisLimitMinTimer, TICK_TIMER_HZ>;
type WidthAxisLimitMaxPin = Pin<'F', 14, Input<PullUp>>; // D4
type WidthAxisLimitMaxTimer = SubTimer<TICK_TIMER_HZ>;
type WidthAxisLimitMax =
    SwitchDevice<WidthAxisLimitMaxPin, SwitchActiveLow, WidthAxisLimitMaxTimer, TICK_TIMER_HZ>;

/* axes */

const LENGTH_AXIS_TIMER_HZ: u32 = 1_000_000;
type LengthAxisStepPin = Pin<'G', 1, Output<PushPull>>;
type LengthAxisDirPin = Pin<'F', 9, Output<PushPull>>;
type LengthAxisTimer = Counter<pac::TIM3, LENGTH_AXIS_TIMER_HZ>;

const WIDTH_AXIS_TIMER_HZ: u32 = 1_000_000;
type WidthAxisStepPin = Pin<'F', 7, Output<PushPull>>;
type WidthAxisDirPin = Pin<'F', 8, Output<PushPull>>;
type WidthAxisTimer = Counter<pac::TIM4, WIDTH_AXIS_TIMER_HZ>;

/* spindle */

type MainSpindleSerial = Serial<pac::USART2, (gpio::PD5<Alternate<7>>, gpio::PD6<Alternate<7>>)>;
type MainSpindleDriver = SpindleDriverJmcHsv57<MainSpindleSerial>;

#[entry]
fn main() -> ! {
    init_heap();

    defmt::println!("Init!");

    let p = pac::Peripherals::take().unwrap();

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

    defmt::println!(
        "Stepper timer clock: {}",
        <pac::TIM3 as BusTimerClock>::timer_clock(&clocks)
    );

    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();
    let gpiod = p.GPIOD.split();
    let gpiof = p.GPIOF.split();
    let gpiog = p.GPIOG.split();

    let tick_timer_device: TickTimerDevice = p.TIM5.counter_us(&clocks);
    let mut super_timer: TickTimer = SuperTimer::new(tick_timer_device, TICK_TIMER_MAX);

    let user_button_pin: UserButtonPin = gpioc.pc13.into_floating_input();
    let user_button_timer: UserButtonTimer = super_timer.sub();
    let mut user_button: UserButton =
        SwitchDevice::new_active_high(user_button_pin, user_button_timer);

    let green_led_pin: GreenLedPin = gpiob.pb0.into_push_pull_output();
    let green_led_timer: GreenLedTimer = super_timer.sub();
    let green_led = LedDevice::new(green_led_pin, green_led_timer);

    let blue_led_pin: BlueLedPin = gpiob.pb7.into_push_pull_output();
    let blue_led_timer: BlueLedTimer = super_timer.sub();
    let blue_led = LedDevice::new(blue_led_pin, blue_led_timer);

    let red_led_pin: RedLedPin = gpiob.pb14.into_push_pull_output();
    let red_led_timer: RedLedTimer = super_timer.sub();
    let red_led = LedDevice::new(red_led_pin, red_led_timer);

    let max_acceleration_in_millimeters_per_sec_per_sec = 5_f64;

    // https://www.makerstore.com.au/product/gear-m1/
    let length_axis_steps_per_revolution = 6400_f64;
    let length_axis_millimeters_per_revolution = 125.66_f64;
    let length_axis_steps_per_millimeter =
        length_axis_steps_per_revolution * (1_f64 / length_axis_millimeters_per_revolution);

    defmt::println!(
        "Length axis steps per mm: {}",
        length_axis_steps_per_millimeter
    );

    let length_axis_dir_pin: LengthAxisDirPin = gpiof.pf9.into_push_pull_output();
    let length_axis_step_pin: LengthAxisStepPin = gpiog.pg1.into_push_pull_output();
    let length_axis_timer: LengthAxisTimer = p.TIM3.counter(&clocks);
    let length_axis_limit_min_pin: LengthAxisLimitMinPin = gpiog.pg9.into_pull_up_input();
    let length_axis_limit_min_timer: LengthAxisLimitMinTimer = super_timer.sub();
    let length_axis_limit_min: LengthAxisLimitMin =
        SwitchDevice::new_active_low(length_axis_limit_min_pin, length_axis_limit_min_timer);
    let length_axis_limit_max_pin: LengthAxisLimitMaxPin = gpiog.pg14.into_pull_up_input();
    let length_axis_limit_max_timer: LengthAxisLimitMaxTimer = super_timer.sub();
    let length_axis_limit_max: LengthAxisLimitMax =
        SwitchDevice::new_active_low(length_axis_limit_max_pin, length_axis_limit_max_timer);
    let length_axis = AxisDevice::new_dq542ma(
        length_axis_dir_pin,
        length_axis_step_pin,
        length_axis_timer,
        max_acceleration_in_millimeters_per_sec_per_sec,
        length_axis_steps_per_millimeter,
        length_axis_limit_min,
        length_axis_limit_max,
        AxisLimitSide::Min,
    );

    let width_axis_steps_per_revolution = 6400_f64;
    let width_axis_leadscrew_starts = 4_f64;
    let width_axis_leadscrew_pitch = 2_f64;
    let width_axis_millimeters_per_revolution =
        width_axis_leadscrew_starts * width_axis_leadscrew_pitch;
    let width_axis_steps_per_millimeter =
        width_axis_steps_per_revolution / width_axis_millimeters_per_revolution;

    defmt::println!(
        "Width axis steps per mm: {}",
        width_axis_steps_per_millimeter
    );

    let width_axis_dir_pin: WidthAxisDirPin = gpiof.pf8.into_push_pull_output();
    let width_axis_step_pin: WidthAxisStepPin = gpiof.pf7.into_push_pull_output();
    let width_axis_timer: WidthAxisTimer = p.TIM4.counter(&clocks);
    let width_axis_limit_min_pin: WidthAxisLimitMinPin = gpiof.pf15.into_pull_up_input();
    let width_axis_limit_min_timer: WidthAxisLimitMinTimer = super_timer.sub();
    let width_axis_limit_min: WidthAxisLimitMin =
        SwitchDevice::new_active_low(width_axis_limit_min_pin, width_axis_limit_min_timer);
    let width_axis_limit_max_pin: WidthAxisLimitMaxPin = gpiof.pf14.into_pull_up_input();
    let width_axis_limit_max_timer: WidthAxisLimitMaxTimer = super_timer.sub();
    let width_axis_limit_max: WidthAxisLimitMax =
        SwitchDevice::new_active_low(width_axis_limit_max_pin, width_axis_limit_max_timer);
    let width_axis = AxisDevice::new_dq542ma(
        width_axis_dir_pin,
        width_axis_step_pin,
        width_axis_timer,
        max_acceleration_in_millimeters_per_sec_per_sec,
        width_axis_steps_per_millimeter,
        width_axis_limit_min,
        width_axis_limit_max,
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

    let mut robot = RobotBuilder::new()
        .with_leds(LedSet::new(green_led, blue_led, red_led))
        .with_axes(AxisSet::new(length_axis, width_axis))
        .with_spindles(SpindleSet::new(main_spindle))
        .build()
        .with_run_commands(&get_run_commands())
        .with_start_commands(&get_start_commands())
        .with_stop_commands(&get_stop_commands())
        .build::<ACTIVE_COMMANDS_COUNT>();

    let mut iwdg = watchdog::IndependentWatchdog::new(p.IWDG);

    iwdg.start(2.millis());

    super_timer.setup().expect("Failed to setup super time");
    loop {
        super_timer.tick().expect("Failed to tick super timer");

        if let Some(user_button_update) = user_button.sense().expect("Error reading user button") {
            if let SwitchStatus::On = user_button_update.status {
                robot.toggle();
            }
        }

        if let Poll::Ready(Err(err)) = robot.poll() {
            defmt::println!("Unexpected error: {}", Debug2Format(&err));

            robot.stop();
            loop {
                match robot.poll() {
                    _ => {}
                }
            }
        }

        iwdg.feed();
    }
}
