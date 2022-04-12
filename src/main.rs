#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

use gridbot as _;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [USART1])]
mod app {
    use alloc_cortex_m::CortexMHeap;
    use core::task::Poll;
    use fugit::ExtU32;
    use stm32f7xx_hal::{pac, prelude::*, timer::monotonic::MonoTimerUs, watchdog};

    use gridbot::{
        actuators::led::LedBlinkMessage,
        command::{Command, CommandActuators, CommandActuatorsResources},
    };

    #[global_allocator]
    static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        actuators: CommandActuators,
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

        let actuators = CommandActuators::new(CommandActuatorsResources {
            GPIOB: ctx.device.GPIOB,
            TIM3: ctx.device.TIM3,
            TIM4: ctx.device.TIM4,
            TIM5: ctx.device.TIM5,
            clocks: &clocks,
        });

        let iwdg = watchdog::IndependentWatchdog::new(ctx.device.IWDG);

        (Shared {}, Local { iwdg, actuators }, init::Monotonics(mono))
    }

    #[idle(local = [actuators, iwdg])]
    fn idle(ctx: idle::Context) -> ! {
        defmt::println!("Hello, world!");

        let iwdg = ctx.local.iwdg;
        iwdg.start(2.millis());

        let actuators = ctx.local.actuators;

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

            let mut future = actuators.run(command);

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
