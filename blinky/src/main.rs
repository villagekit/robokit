#![no_main]
#![no_std]

use blinky as _;

use core::task::Poll;
use cortex_m_rt::entry;
use defmt::Debug2Format;
use fugit::ExtU32;
use robokit::{
    actuator_set, Command, LedAction, LedDevice, RobotBuilder, Sensor, SuperTimer, SwitchDevice,
    SwitchStatus,
};
use stm32f7xx_hal::{pac, prelude::*};

use blinky::init_heap;

const TICK_TIMER_HZ: u32 = 1_000_000;
const ACTIVE_COMMANDS_COUNT: usize = 1;

actuator_set!(
    Led { Green, Blue, Red },
    LedAction<TICK_TIMER_HZ>,
    LedId,
    LedSet,
    LedSetError
);

fn get_run_commands<const TIMER_HZ: u32>() -> [Command<TIMER_HZ, LedId, (), ()>; 6] {
    [
        Command::Led(
            LedId::Green,
            LedAction::Blink {
                duration: 50.millis(),
            },
        ),
        Command::Led(
            LedId::Blue,
            LedAction::Blink {
                duration: 100.millis(),
            },
        ),
        Command::Led(
            LedId::Red,
            LedAction::Blink {
                duration: 200.millis(),
            },
        ),
        Command::Led(
            LedId::Red,
            LedAction::Blink {
                duration: 50.millis(),
            },
        ),
        Command::Led(
            LedId::Blue,
            LedAction::Blink {
                duration: 100.millis(),
            },
        ),
        Command::Led(
            LedId::Green,
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

    let green_led_pin = gpiob.pb0.into_push_pull_output();
    let green_led_timer = super_timer.sub();
    let green_led = LedDevice::new(green_led_pin, green_led_timer);

    let blue_led_pin = gpiob.pb7.into_push_pull_output();
    let blue_led_timer = super_timer.sub();
    let blue_led = LedDevice::new(blue_led_pin, blue_led_timer);

    let red_led_pin = gpiob.pb14.into_push_pull_output();
    let red_led_timer = super_timer.sub();
    let red_led = LedDevice::new(red_led_pin, red_led_timer);

    let mut robot = RobotBuilder::new()
        .with_leds(LedSet::new(green_led, blue_led, red_led))
        .build()
        .with_run_commands(&get_run_commands())
        .build::<ACTIVE_COMMANDS_COUNT>();

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
        }
    }
}
