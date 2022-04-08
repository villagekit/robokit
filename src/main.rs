#![no_main]
#![no_std]

use gridbot as _;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [USART1])]
mod app {
    use fugit::ExtU32;
    use stm32f7xx_hal::{
        gpio::{Output, PB0},
        pac,
        prelude::*,
        timer::monotonic::MonoTimerUs,
        watchdog,
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PB0<Output>,
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
        let led = gpiob.pb0.into_push_pull_output();

        let iwdg = watchdog::IndependentWatchdog::new(ctx.device.IWDG);

        tick::spawn().ok();

        (Shared {}, Local { led, iwdg }, init::Monotonics(mono))
    }

    #[idle(local = [iwdg])]
    fn idle(ctx: idle::Context) -> ! {
        defmt::println!("Hello, world!");

        let iwdg = ctx.local.iwdg;

        iwdg.start(2.millis());

        loop {
            iwdg.feed();
        }
    }

    #[task(local = [led])]
    fn tick(ctx: tick::Context) {
        ctx.local.led.toggle();

        tick::spawn_after(1.secs()).ok();
    }
}
