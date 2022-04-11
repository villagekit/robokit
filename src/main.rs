#![no_main]
#![no_std]

use gridbot as _;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [USART1])]
mod app {
    use fugit::ExtU32;
    use stm32f7xx_hal::{
        gpio::{Output, Pin, PushPull},
        pac,
        prelude::*,
        timer::{counter::CounterMs, monotonic::MonoTimerUs, TimerExt},
        watchdog,
    };

    use gridbot::actuator::{Command, Waitable};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        green_led: gridbot::actuator::Led<Pin<'B', 0, Output<PushPull>>, CounterMs<pac::TIM3>>,
        blue_led: gridbot::actuator::Led<Pin<'B', 7, Output<PushPull>>, CounterMs<pac::TIM4>>,
        red_led: gridbot::actuator::Led<Pin<'B', 14, Output<PushPull>>, CounterMs<pac::TIM5>>,
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
        let green_led = gridbot::actuator::Led {
            pin: gpiob.pb0.into_push_pull_output(),
            timer: ctx.device.TIM3.counter_ms(&clocks),
        };
        let blue_led = gridbot::actuator::Led {
            pin: gpiob.pb7.into_push_pull_output(),
            timer: ctx.device.TIM4.counter_ms(&clocks),
        };
        let red_led = gridbot::actuator::Led {
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

    #[idle(local = [green_led, blue_led, red_led, iwdg])]
    fn idle(ctx: idle::Context) -> ! {
        defmt::println!("Hello, world!");

        let iwdg = ctx.local.iwdg;
        iwdg.start(2.millis());

        let led = ctx.local.green_led;

        loop {
            let message = gridbot::actuator::LedBlink { duration: 1.secs() };
            led.command(message);

            loop {
                match led.wait() {
                    Err(nb::Error::Other(_err)) => {
                        break;
                    }
                    Err(nb::Error::WouldBlock) => {}
                    Ok(_value) => {
                        break;
                    }
                }

                iwdg.feed();
            }
        }
    }
}
