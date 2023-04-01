use alloc::boxed::Box;
use core::fmt::Debug;

pub type Error = Box<dyn Debug>;
