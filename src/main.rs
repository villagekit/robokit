#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

use gridbot as _;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [USART1])]
mod app {
    extern crate alloc;

    use alloc::boxed::Box;
    use alloc_cortex_m::CortexMHeap;
    use core::task::Poll;
    use fugit::ExtU32;
    use stm32f7xx_hal::{
        gpio::{Output, Pin, PushPull},
        pac,
        prelude::*,
        timer::{counter::CounterMs, monotonic::MonoTimerUs, TimerExt},
        watchdog,
    };

    use gridbot::{
        actuator::{Actuator, Future},
        actuators::led::{Led, LedBlinkMessage},
        command::Command,
    };

    #[global_allocator]
    static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

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
        // Initialize the allocator BEFORE you use it
        {
            use core::mem::MaybeUninit;
            const HEAP_SIZE: usize = 1024;
            static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
            unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
        }

        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        let mono = ctx.device.TIM2.monotonic_us(&clocks);

        let gpiob = ctx.device.GPIOB.split();
        let green_led = Led::new(
            gpiob.pb0.into_push_pull_output(),
            ctx.device.TIM3.counter_ms(&clocks),
        );
        let blue_led = Led::new(
            gpiob.pb7.into_push_pull_output(),
            ctx.device.TIM4.counter_ms(&clocks),
        );
        let red_led = Led::new(
            gpiob.pb14.into_push_pull_output(),
            ctx.device.TIM5.counter_ms(&clocks),
        );

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

        let green_led = ctx.local.green_led;
        let blue_led = ctx.local.blue_led;
        let red_led = ctx.local.red_led;

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

            let mut future: Box<dyn Future> = match command {
                Command::GreenLed(message) => Box::new(green_led.command(message)),
                Command::BlueLed(message) => Box::new(blue_led.command(message)),
                Command::RedLed(message) => Box::new(red_led.command(message)),
            };

            loop {
                match future.poll() {
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
}
