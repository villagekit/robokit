use core::task::Poll;

use crate::error::Error;

pub trait ActorReceive {
    type Message;

    fn receive(&mut self, message: &Self::Message);
}

pub trait ActorPoll {
    fn poll(&mut self) -> Poll<Result<(), Error>>;
}
