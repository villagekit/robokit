#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [USART1])]
mod app {
    use stm32f7xx_hal::{
        gpio::{Output, PB0},
        pac,
        prelude::*,
        timer::monotonic::MonoTimerUs,
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PB0<Output>,
    }

    #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimerUs<pac::TIM2>;

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        let gpiob = ctx.device.GPIOB.split();
        let led = gpiob.pb0.into_push_pull_output();

        let mono = ctx.device.TIM2.monotonic_us(&clocks);
        tick::spawn().ok();
        (Shared {}, Local { led }, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }

    #[task(local = [led])]
    fn tick(ctx: tick::Context) {
        tick::spawn_after(1.secs()).ok();
        ctx.local.led.toggle();
    }
}
