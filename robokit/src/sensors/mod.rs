pub mod switch;

use crate::error::Error;

pub trait Sensor {
    type Message;
    type Error: Error;

    fn sense(&mut self) -> Result<Option<Self::Message>, Self::Error>;
}
