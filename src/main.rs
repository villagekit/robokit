#![no_main]
#![no_std]

use gridbot as _;

use core::task::Poll;
use cortex_m_rt::entry;
use defmt::Debug2Format;
use fugit::ExtU32;
use stm32f7xx_hal::{
    gpio::{Floating, Input, Pin},
    pac,
    prelude::*,
    rcc::BusTimerClock,
    serial::{Config as SerialConfig, DataBits as SerialDataBits, Parity as SerialParity, Serial},
    timer::Counter,
    watchdog,
};

use gridbot::{
    actor::{ActorPoll, ActorReceive, ActorSense},
    command::{CommandCenter, CommandCenterResources},
    init_heap,
    machine::{Machine, StopMessage, ToggleMessage},
    sensors::switch::{Switch, SwitchActiveHigh, SwitchError, SwitchStatus},
    timer::{setup as timer_setup, tick as timer_tick, SubTimer, TICK_TIMER_HZ},
};

pub const TICK_TIMER_MAX: u32 = u32::MAX;
pub type TickTimer = Counter<pac::TIM5, TICK_TIMER_HZ>;

type UserButtonPin = Pin<'C', 13, Input<Floating>>;
type UserButtonTimer = SubTimer;
// type UserButtonError = SwitchError<<UserButtonPin as InputPin>::Error>;
type UserButton = Switch<UserButtonPin, SwitchActiveHigh, UserButtonTimer, TICK_TIMER_HZ>;

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
    let mut user_button: UserButton = Switch::new(user_button_pin, user_button_timer);

    let green_led_pin = gpiob.pb0.into_push_pull_output();
    let green_led_timer = SubTimer::new();
    let blue_led_pin = gpiob.pb7.into_push_pull_output();
    let blue_led_timer = SubTimer::new();
    let red_led_pin = gpiob.pb14.into_push_pull_output();
    let red_led_timer = SubTimer::new();

    defmt::println!(
        "Stepper timer clock: {}",
        <pac::TIM3 as BusTimerClock>::timer_clock(&clocks)
    );

    let x_axis_dir_pin = gpiog.pg9.into_push_pull_output();
    let x_axis_step_pin = gpiog.pg14.into_push_pull_output();
    let x_axis_timer = p.TIM3.counter(&clocks);

    let x_axis_limit_min_pin = gpiof.pf15.into_floating_input();
    let x_axis_limit_min_timer = SubTimer::new();
    let x_axis_limit_max_pin = gpioe.pe13.into_floating_input();
    let x_axis_limit_max_timer = SubTimer::new();

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
