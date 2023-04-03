use alloc::boxed::Box;
use core::fmt::{Debug, Formatter, Result as FmtResult};

pub trait Error: Debug {}

impl<E: Error + 'static> From<E> for Box<dyn Error> {
    fn from(error: E) -> Box<dyn Error> {
        Box::new(error)
    }
}

pub struct BoxError(Box<dyn Error>);

impl Debug for BoxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}
impl Error for BoxError {}

impl From<Box<dyn Error>> for BoxError {
    fn from(error: Box<dyn Error>) -> BoxError {
        BoxError(error)
    }
}
