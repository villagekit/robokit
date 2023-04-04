#![no_std]
#![no_main]

use gridbot_tahi as _; // memory layout + panic handler + others

// See https://crates.io/crates/defmt-test/0.3.0 for more documentation (e.g. about the 'state'
// feature)
#[defmt_test::tests]
mod tests {
    use defmt::assert;

    use gridbot_tahi::init_heap;

    #[init]
    fn init() {
        init_heap();
    }

    #[test]
    fn it_works() {
        assert!(true)
    }
}
