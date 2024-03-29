#![no_main]
#![no_std]

extern crate alloc;

pub mod actuators;
pub mod commands;

use embedded_alloc::Heap;

use defmt_rtt as _; // global logger

use stm32f7xx_hal as _; // memory layout

use panic_probe as _;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[global_allocator]
static HEAP: Heap = Heap::empty();

pub fn init_heap() {
    // Initialize the allocator BEFORE you use it
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 1024;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}

// defmt-test 0.3.0 has the limitation that this `#[tests]` attribute can only be used
// once within a crate. the module can be in any file but there can only be at most
// one `#[tests]` module in this library crate
#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use defmt::assert_eq;

    use robokit::util;

    #[test]
    fn i16_to_u16() {
        assert_eq!(util::i16_to_u16(0), 0);
        assert_eq!(util::i16_to_u16(1), 1);
        assert_eq!(util::i16_to_u16(2), 2);
        assert_eq!(util::i16_to_u16(32767), 32767);
        assert_eq!(util::i16_to_u16(-1), 65535);
        assert_eq!(util::i16_to_u16(-2), 65534);
        assert_eq!(util::i16_to_u16(-32768), 32768);
    }

    #[test]
    fn u16_to_i16() {
        assert_eq!(util::u16_to_i16(0), 0);
        assert_eq!(util::u16_to_i16(1), 1);
        assert_eq!(util::u16_to_i16(2), 2);
        assert_eq!(util::u16_to_i16(32767), 32767);
        assert_eq!(util::u16_to_i16(65535), -1);
        assert_eq!(util::u16_to_i16(65534), -2);
        assert_eq!(util::u16_to_i16(32768), -32768);
    }
}
