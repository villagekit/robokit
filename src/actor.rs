use core::task::Poll;

use crate::error::Error;

pub trait ActorReceive<'a> {
    type Message;

    fn receive(&'a mut self, message: &Self::Message);
}

pub trait ActorFuture<'a> {
    fn poll(&'a mut self) -> Poll<Result<(), Error>>;
}
