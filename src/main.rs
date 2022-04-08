#![no_main]
#![no_std]

use gridbot as _;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [USART1])]
mod app {
    use fugit::ExtU32;
    use stm32f7xx_hal::{
        gpio::{Output, PB0, PB14, PB7},
        pac,
        prelude::*,
        timer::monotonic::MonoTimerUs,
        watchdog,
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        green_led: PB0<Output>,
        blue_led: PB7<Output>,
        red_led: PB14<Output>,
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
        let green_led = gpiob.pb0.into_push_pull_output();
        let blue_led = gpiob.pb7.into_push_pull_output();
        let red_led = gpiob.pb14.into_push_pull_output();

        let iwdg = watchdog::IndependentWatchdog::new(ctx.device.IWDG);

        green_led_tick::spawn().ok();
        blue_led_tick::spawn().ok();
        red_led_tick::spawn().ok();

        (
            Shared {},
            Local {
                green_led,
                blue_led,
                red_led,
                iwdg,
            },
            init::Monotonics(mono),
        )
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

    #[task(local = [green_led])]
    fn green_led_tick(ctx: green_led_tick::Context) {
        ctx.local.green_led.toggle();

        green_led_tick::spawn_after(1.secs()).ok();
    }

    #[task(local = [blue_led])]
    fn blue_led_tick(ctx: blue_led_tick::Context) {
        ctx.local.blue_led.toggle();

        blue_led_tick::spawn_after(2.secs()).ok();
    }

    #[task(local = [red_led])]
    fn red_led_tick(ctx: red_led_tick::Context) {
        ctx.local.red_led.toggle();

        red_led_tick::spawn_after(4.secs()).ok();
    }
}
