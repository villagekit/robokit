#![no_main]
#![no_std]

use blinky as _;

use core::task::Poll;
use cortex_m_rt::entry;
use defmt::Debug2Format;
use fugit::ExtU32;
use robokit::{actuators::led::LedAction, runner::Command};
use robokit::{
    actuators::led::LedDevice,
    robot::RobotBuilder,
    sensors::{
        switch::{SwitchDevice, SwitchStatus},
        Sensor,
    },
    timer::SuperTimer,
};
use stm32f7xx_hal::{pac, prelude::*};

use blinky::init_heap;

const TICK_TIMER_HZ: u32 = 1_000_000;
const RUN_COMMANDS_COUNT: usize = 6;
const START_COMMANDS_COUNT: usize = 0;
const STOP_COMMANDS_COUNT: usize = 0;
const LEDS_COUNT: usize = 4;
const AXES_COUNT: usize = 2;
const SPINDLES_COUNT: usize = 2;
const ACTIVE_COMMANDS_COUNT: usize = 1;

pub fn get_run_commands<const TIMER_HZ: u32>() -> [Command<'static, TIMER_HZ>; 6] {
    [
        Command::Led(
            "green",
            LedAction::Blink {
                duration: 50.millis(),
            },
        ),
        Command::Led(
            "blue",
            LedAction::Blink {
                duration: 100.millis(),
            },
        ),
        Command::Led(
            "red",
            LedAction::Blink {
                duration: 200.millis(),
            },
        ),
        Command::Led(
            "red",
            LedAction::Blink {
                duration: 50.millis(),
            },
        ),
        Command::Led(
            "blue",
            LedAction::Blink {
                duration: 100.millis(),
            },
        ),
        Command::Led(
            "green",
            LedAction::Blink {
                duration: 200.millis(),
            },
        ),
    ]
}

#[entry]
fn main() -> ! {
    init_heap();

    defmt::println!("Init!");

    let p = pac::Peripherals::take().unwrap();

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();

    let tick_timer = p.TIM5.counter_us(&clocks);
    let mut super_timer = SuperTimer::new(tick_timer, u32::MAX);

    let user_button_pin = gpioc.pc13.into_floating_input();
    let user_button_timer = super_timer.sub();
    let mut user_button = SwitchDevice::new_active_high(user_button_pin, user_button_timer);

    let mut robot_builder: RobotBuilder<
        '_,
        TICK_TIMER_HZ,
        RUN_COMMANDS_COUNT,
        START_COMMANDS_COUNT,
        STOP_COMMANDS_COUNT,
        LEDS_COUNT,
        AXES_COUNT,
        SPINDLES_COUNT,
        ACTIVE_COMMANDS_COUNT,
    > = RobotBuilder::new();

    let green_led_pin = gpiob.pb0.into_push_pull_output();
    let green_led_timer = super_timer.sub();
    let green_led = LedDevice::new(green_led_pin, green_led_timer);
    robot_builder.add_led("green", green_led).unwrap();

    let blue_led_pin = gpiob.pb7.into_push_pull_output();
    let blue_led_timer = super_timer.sub();
    let blue_led = LedDevice::new(blue_led_pin, blue_led_timer);
    robot_builder.add_led("blue", blue_led).unwrap();

    let red_led_pin = gpiob.pb14.into_push_pull_output();
    let red_led_timer = super_timer.sub();
    let red_led = LedDevice::new(red_led_pin, red_led_timer);
    robot_builder.add_led("red", red_led).unwrap();

    robot_builder.set_run_commands(&get_run_commands()).unwrap();

    let mut robot = robot_builder.build().expect("Error validating robot");

    loop {
        super_timer.tick().expect("Error ticking super timer");

        if let Some(user_button_update) = user_button.sense().expect("Error reading user button") {
            if let SwitchStatus::On = user_button_update.status {
                robot.toggle();
            }
        }

        if let Poll::Ready(Err(err)) = robot.poll() {
            defmt::println!("Unexpected error: {}", Debug2Format(&err));

            robot.stop();
        }
    }
}
