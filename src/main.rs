#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use gridbot as _;

use core::task::Poll;
use cortex_m_rt::entry;
use fugit::ExtU32;
use stm32f7xx_hal::{pac, prelude::*, watchdog};

use gridbot::{
    actor::{ActorFuture, ActorReceive},
    actuators::led::LedBlinkMessage,
    command::{Command, CommandCenter, CommandCenterResources},
};

#[entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    let device = pac::Peripherals::take().unwrap();
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    let mut command_center = CommandCenter::new(CommandCenterResources {
        GPIOB: device.GPIOB,
        TIM3: device.TIM3,
        TIM4: device.TIM4,
        TIM5: device.TIM5,
        clocks: &clocks,
    });

    let mut iwdg = watchdog::IndependentWatchdog::new(device.IWDG);
    iwdg.start(2.millis());

    let commands = [
        Command::GreenLed(LedBlinkMessage {
            duration: 1000.millis(),
        }),
        Command::BlueLed(LedBlinkMessage {
            duration: 2000.millis(),
        }),
        Command::RedLed(LedBlinkMessage {
            duration: 500.millis(),
        }),
    ];
    let mut command_index = 0;

    loop {
        let command = &commands[command_index];

        command_center.receive(command);

        loop {
            match command_center.poll() {
                Poll::Ready(Err(err)) => {
                    panic!("Unexpected error: {:?}", err);
                }
                Poll::Ready(Ok(())) => {
                    break;
                }
                Poll::Pending => {}
            }

            iwdg.feed();
        }

        command_index = (command_index + 1) % commands.len();
    }
}
