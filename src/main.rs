#![no_main]
#![no_std]

use gridbot as _;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [USART1])]
mod app {
    use core::task::Poll;
    use defmt::Debug2Format;
    use fugit::ExtU32;
    use stm32f7xx_hal::{
        gpio::{Floating, Input, Pin},
        pac,
        prelude::*,
        rcc::BusTimerClock,
        serial::{
            Config as SerialConfig, DataBits as SerialDataBits, Parity as SerialParity, Serial,
        },
        timer::{monotonic::MonoTimerUs, Counter},
        watchdog,
    };

    use gridbot::{
        actor::{ActorPoll, ActorReceive, ActorSense},
        command::{CommandCenter, CommandCenterResources},
        machine::{Machine, StopMessage, ToggleMessage},
        sensors::switch::{Switch, SwitchActiveHigh, SwitchError, SwitchStatus},
        timer::{setup as timer_setup, tick as timer_tick, SubTimer, TICK_TIMER_HZ},
    };

    pub const TICK_TIMER_MAX: u32 = u32::MAX;
    pub type TickTimer = Counter<pac::TIM5, TICK_TIMER_HZ>;

    type UserButtonPin = Pin<'C', 13, Input<Floating>>;
    type UserButtonTimer = SubTimer;
    // type UserButtonError = SwitchError<<UserButtonPin as InputPin>::Error>;
    type UserButton = Switch<UserButtonPin, SwitchActiveHigh, UserButtonTimer, TICK_TIMER_HZ>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        tick_timer: TickTimer,
        user_button: UserButton,
        machine: Machine,
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

        let gpiob = ctx.device.GPIOB.split();
        let gpioc = ctx.device.GPIOC.split();
        let gpiod = ctx.device.GPIOD.split();
        let gpiof = ctx.device.GPIOF.split();
        let gpiog = ctx.device.GPIOG.split();

        let mut tick_timer = ctx.device.TIM5.counter_us(&clocks);
        timer_setup(&mut tick_timer, TICK_TIMER_MAX).unwrap();

        let user_button_pin = gpioc.pc13.into_floating_input();
        let user_button_timer = SubTimer::new();
        let user_button = Switch::new(user_button_pin, user_button_timer);

        let green_led_pin = gpiob.pb0.into_push_pull_output();
        let green_led_timer = SubTimer::new();
        let blue_led_pin = gpiob.pb7.into_push_pull_output();
        let blue_led_timer = SubTimer::new();
        let red_led_pin = gpiob.pb14.into_push_pull_output();
        let red_led_timer = SubTimer::new();

        defmt::println!(
            "Stepper timer clock: {}",
            <pac::TIM3 as BusTimerClock>::timer_clock(&clocks)
        );

        let x_axis_dir_pin = gpiog.pg9.into_push_pull_output();
        let x_axis_step_pin = gpiog.pg14.into_push_pull_output();
        let x_axis_timer = ctx.device.TIM3.counter(&clocks);

        let x_axis_limit_min_pin = gpiof.pf15.into_pull_up_input();
        let x_axis_limit_min_timer = SubTimer::new();
        let x_axis_limit_max_pin = gpiof.pf14.into_pull_up_input();
        let x_axis_limit_max_timer = SubTimer::new();

        let main_spindle_serial_tx = gpiod.pd5.into_alternate();
        let main_spindle_serial_rx = gpiod.pd6.into_alternate();
        let main_spindle_serial = Serial::new(
            ctx.device.USART2,
            (main_spindle_serial_tx, main_spindle_serial_rx),
            &clocks,
            SerialConfig {
                baud_rate: 57600.bps(),
                data_bits: SerialDataBits::Bits9,
                parity: SerialParity::ParityEven,
                ..Default::default()
            },
        );

        let command_center = CommandCenter::new(CommandCenterResources {
            green_led_pin,
            green_led_timer,
            blue_led_pin,
            blue_led_timer,
            red_led_pin,
            red_led_timer,
            x_axis_dir_pin,
            x_axis_step_pin,
            x_axis_timer,
            x_axis_limit_min_pin,
            x_axis_limit_min_timer,
            x_axis_limit_max_pin,
            x_axis_limit_max_timer,
            main_spindle_serial,
        });
        let machine = Machine::new(command_center);

        let iwdg = watchdog::IndependentWatchdog::new(ctx.device.IWDG);

        (
            Shared {},
            Local {
                tick_timer,
                user_button,
                iwdg,
                machine,
            },
            init::Monotonics(mono),
        )
    }

    #[idle(local = [tick_timer, user_button, machine, iwdg])]
    fn idle(ctx: idle::Context) -> ! {
        let tick_timer = ctx.local.tick_timer;
        let user_button = ctx.local.user_button;
        let iwdg = ctx.local.iwdg;
        let machine = ctx.local.machine;

        iwdg.start(2.millis());

        loop {
            timer_tick(tick_timer, TICK_TIMER_MAX).unwrap();

            if let Some(user_button_update) =
                user_button.sense().expect("Error reading user button")
            {
                if let SwitchStatus::On = user_button_update.status {
                    machine.receive(&ToggleMessage {});
                }
            }

            if let Poll::Ready(Err(err)) = machine.poll() {
                defmt::println!("Unexpected error: {}", Debug2Format(&err));

                machine.receive(&StopMessage {});
                loop {
                    match machine.poll() {
                        _ => {}
                    }
                }
            }

            iwdg.feed();
        }
    }
}
