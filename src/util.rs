// two's complement
pub fn i16_to_u16(num: i16) -> u16 {
    defmt::println!("num: {}", num);
    if num == 0 {
        0
    } else if num > 0 {
        num as u16
    } else {
        u16::MAX - ((-1 - num) as u16)
    }
}

// two's complement
pub fn u16_to_i16(num: u16) -> i16 {
    defmt::println!("num: {}", num);
    if num == 0 {
        0
    } else if num <= i16::MAX as u16 {
        num as i16
    } else {
        -((u16::MAX - num) as i16) - 1
    }
}
