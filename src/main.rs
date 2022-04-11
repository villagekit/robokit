#![no_std]
#![no_main]

use stm32f7xx_hal as hal;

use gridbot;

// How often our blinky task wakes up (1/2 our blink frequency).
const PERIOD: core::time::Duration = core::time::Duration::from_millis(500);

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let p = hal::pac::Peripherals::take().unwrap();

    let rcc = ctx.device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(16.MHz()).freeze();

    let gpiob = p.GPIOB.split();
    let led = gpiob.pb0.into_push_pull_output();

    let blink = async {
        let mut gate = lilos::exec::PeriodicGate::new(PERIOD);

        loop {
            led.toggle();
            gate.next_time().await;
            led.toggle();
            gate.next_time().await;
        }
    };
    pin_utils::pin_mut!(blink);

    lilos::time::initialize_sys_tick(&mut cp.SYST, 16_000_000);
    lilos::exec::run_tasks(&mut [blink], lilos::exec::ALL_TASKS)
}
