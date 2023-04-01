pub mod switch;

use crate::error::Error;

pub trait Sensor<Message> {
    fn sense(&mut self) -> Result<Option<Message>, Error>;
}
