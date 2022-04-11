#![no_main]
#![no_std]

use gridbot;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [USART1])]
mod app {
    use stm32f7xx_hal::{gpio::PB0, pac, prelude::*, timer::monotonic::MonoTimerUs};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: gridbot::actuator::Led<PB0, pac::TIM3>,
    }

    #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimerUs<pac::TIM2>;

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        let gpiob = ctx.device.GPIOB.split();
        let led = gridbot::actuator::Led {
            pin: gpiob.pb0.into_push_pull_output(),
            timer: ctx.device.TIM3.monotonic_us(&clocks),
        };

        let mono = ctx.device.TIM2.monotonic_us(&clocks);
        (Shared {}, Local { led }, init::Monotonics(mono))
    }

    #[idle]
    fn idle(ctx: idle::Context) -> ! {
        defmt::println!("Hello, world!");

        loop {
            let led = ctx.local.led;
            let command = gridbot::actuator::LedBlink { duration: 10.Hz() };
            led.command(command);
            nb::block!(led.wait());
        }
    }
}
