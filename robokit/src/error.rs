use core::fmt::Debug;

pub trait Error: Debug {}

impl<E: Debug> Error for E {}
