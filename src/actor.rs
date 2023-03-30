use core::fmt::Debug;
use core::task::Poll;

// ActorReceive trait inspired by https://github.com/rtic-rs/rfcs/pull/0052
pub trait ActorReceive<Message> {
    fn receive(&mut self, message: &Message);
}

// ActorPoll trait inspired by https://docs.rs/stepper
pub trait ActorPoll {
    type Error: Debug;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>>;
}

pub trait ActorSense<Message> {
    type Error: Debug;

    fn sense(&mut self) -> Result<Option<Message>, Self::Error>;
}
