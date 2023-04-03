#![no_std]

extern crate alloc;

pub mod actuators;
pub mod error;
pub mod modbus;
pub mod robot;
pub mod runner;
pub mod scheduler;
pub mod sensors;
pub mod timer;
pub mod util;

#[cfg(test)]
mod unit_tests {
    use core::assert_eq;

    use super::util;

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
