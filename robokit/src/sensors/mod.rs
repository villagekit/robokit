pub mod switch;

use core::fmt::Debug;

pub trait Sensor<Message> {
    type Error: Debug;

    fn sense(&mut self) -> Result<Option<Message>, Self::Error>;
}
