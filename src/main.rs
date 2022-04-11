#![no_main]
#![no_std]

use gridbot as _;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [USART1])]
mod app {
    use fugit::ExtU32;
    use fugit_timer::Timer;
    use stm32f7xx_hal::{
        gpio::{Output, Pin, PushPull},
        pac,
        prelude::*,
        timer::{counter::CounterMs, monotonic::MonoTimerUs, TimerExt},
        watchdog,
    };

    use gridbot::actuator::{Commandable, Led, LedBlink, Waitable};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        green_led: Led<Pin<'B', 0, Output<PushPull>>, CounterMs<pac::TIM3>>,
        blue_led: Led<Pin<'B', 7, Output<PushPull>>, CounterMs<pac::TIM4>>,
        red_led: Led<Pin<'B', 14, Output<PushPull>>, CounterMs<pac::TIM5>>,
        iwdg: watchdog::IndependentWatchdog,
    }

    #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimerUs<pac::TIM2>;

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        let mono = ctx.device.TIM2.monotonic_us(&clocks);

        let gpiob = ctx.device.GPIOB.split();
        let green_led = Led {
            pin: gpiob.pb0.into_push_pull_output(),
            timer: ctx.device.TIM3.counter_ms(&clocks),
        };
        let blue_led = Led {
            pin: gpiob.pb7.into_push_pull_output(),
            timer: ctx.device.TIM4.counter_ms(&clocks),
        };
        let red_led = Led {
            pin: gpiob.pb14.into_push_pull_output(),
            timer: ctx.device.TIM5.counter_ms(&clocks),
        };

        let iwdg = watchdog::IndependentWatchdog::new(ctx.device.IWDG);

        (
            Shared {},
            Local {
                iwdg,
                green_led,
                blue_led,
                red_led,
            },
            init::Monotonics(mono),
        )
    }

    enum Command {
        GreenLed(LedBlink),
        BlueLed(LedBlink),
        RedLed(LedBlink),
    }

    #[idle(local = [green_led, blue_led, red_led, iwdg])]
    fn idle(ctx: idle::Context) -> ! {
        defmt::println!("Hello, world!");

        let iwdg = ctx.local.iwdg;
        iwdg.start(2.millis());

        let green_led = ctx.local.green_led;
        let blue_led = ctx.local.blue_led;
        let red_led = ctx.local.red_led;

        let commands = [
            Command::GreenLed(LedBlink { duration: 1.secs() }),
            Command::BlueLed(LedBlink { duration: 1.secs() }),
            Command::RedLed(LedBlink { duration: 1.secs() }),
        ];
        let mut command_index = 0;
        let mut total_index = 0;

        loop {
            let command = &commands[command_index];

            defmt::println!("Total index: {}", total_index);
            defmt::println!("Command index: {}", command_index);

            let waitable: &mut dyn Waitable<Error = void::Void> = match command {
                Command::GreenLed(message) => {
                    green_led.command(message);
                    green_led
                }
                Command::BlueLed(message) => {
                    blue_led.command(message);
                    blue_led
                }
                Command::RedLed(message) => {
                    red_led.command(message);
                    red_led
                }
            };

            loop {
                match waitable.wait() {
                    Err(nb::Error::Other(err)) => {
                        panic!("Unexpected error: {}", err);
                    }
                    Err(nb::Error::WouldBlock) => {}
                    Ok(_value) => {
                        break;
                    }
                }

                iwdg.feed();
            }

            command_index = (command_index + 1) % commands.len();
            total_index += 1;
        }
    }
}
