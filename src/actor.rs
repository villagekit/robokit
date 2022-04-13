use core::task::Poll;
use enum_dispatch::enum_dispatch;

use crate::error::Error;

#[enum_dispatch]
pub trait ActorReceive<Message> {
    fn receive(&mut self, message: &Message);
}

#[enum_dispatch]
pub trait ActorFuture {
    fn poll(&mut self) -> Poll<Result<(), Error>>;
}
