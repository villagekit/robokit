#![no_main]
#![no_std]

use gridbot as _;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [USART1])]
mod app {
    use core::task::Poll;
    use defmt::Debug2Format;
    use fugit::ExtU32;
    use stm32f7xx_hal::{pac, prelude::*, timer::monotonic::MonoTimerUs, watchdog};

    use gridbot::{
        actor::{ActorPoll, ActorReceive},
        actuators::axis::AxisMoveMessage,
        actuators::led::LedBlinkMessage,
        actuators::spindle::{SpindleSetMessage, SpindleStatus},
        command::{Command, CommandCenter, CommandCenterResources},
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        command_center: CommandCenter,
        iwdg: watchdog::IndependentWatchdog,
    }

    #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimerUs<pac::TIM2>;

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::println!("Init!");

        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

        let mono = ctx.device.TIM2.monotonic_us(&clocks);

        let command_center = CommandCenter::new(CommandCenterResources {
            GPIOB: ctx.device.GPIOB,
            GPIOC: ctx.device.GPIOC,
            GPIOD: ctx.device.GPIOD,
            GPIOG: ctx.device.GPIOG,
            TIM3: ctx.device.TIM3,
            TIM9: ctx.device.TIM9,
            TIM10: ctx.device.TIM10,
            TIM11: ctx.device.TIM11,
            USART2: ctx.device.USART2,
            clocks: &clocks,
        });

        let iwdg = watchdog::IndependentWatchdog::new(ctx.device.IWDG);

        (
            Shared {},
            Local {
                iwdg,
                command_center,
            },
            init::Monotonics(mono),
        )
    }

    #[idle(local = [command_center, iwdg])]
    fn idle(ctx: idle::Context) -> ! {
        let iwdg = ctx.local.iwdg;
        iwdg.start(2.millis());

        let command_center = ctx.local.command_center;

        let commands = [
            Command::MainSpindle(SpindleSetMessage {
                status: SpindleStatus::On { rpm: 1000 },
            }),
            Command::GreenLed(LedBlinkMessage {
                duration: 50000.micros(),
            }),
            Command::BlueLed(LedBlinkMessage {
                duration: 50000.micros(),
            }),
            Command::RedLed(LedBlinkMessage {
                duration: 50000.micros(),
            }),
            Command::XAxis(AxisMoveMessage {
                max_velocity_in_millimeters_per_sec: 40_f64,
                distance_in_millimeters: 40_f64,
            }),
            Command::RedLed(LedBlinkMessage {
                duration: 50000.micros(),
            }),
            Command::BlueLed(LedBlinkMessage {
                duration: 50000.micros(),
            }),
            Command::GreenLed(LedBlinkMessage {
                duration: 50000.micros(),
            }),
            Command::XAxis(AxisMoveMessage {
                max_velocity_in_millimeters_per_sec: 40_f64,
                distance_in_millimeters: -40_f64,
            }),
            Command::MainSpindle(SpindleSetMessage {
                status: SpindleStatus::Off,
            }),
            Command::GreenLed(LedBlinkMessage {
                duration: 50000.micros(),
            }),
            Command::BlueLed(LedBlinkMessage {
                duration: 50000.micros(),
            }),
            Command::RedLed(LedBlinkMessage {
                duration: 50000.micros(),
            }),
            Command::BlueLed(LedBlinkMessage {
                duration: 50000.micros(),
            }),
            Command::GreenLed(LedBlinkMessage {
                duration: 50000.micros(),
            }),
        ];
        let mut command_index = 0;

        loop {
            let command = &commands[command_index];

            defmt::println!("Command: {}", command);

            command_center.receive(command);

            loop {
                match command_center.update() {
                    Err(err) => {
                        defmt::panic!("Unexpected sensor error: {:?}", Debug2Format(&err));
                    }
                    Ok(()) => {}
                }

                match command_center.poll() {
                    Poll::Ready(Err(err)) => {
                        defmt::panic!("Unexpected actuator error: {:?}", Debug2Format(&err));
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
}
