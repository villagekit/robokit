use core::task::Poll;

// ActorReceive trait inspired by https://github.com/rtic-rs/rfcs/pull/0052
pub trait ActorReceive<Message> {
    fn receive(&mut self, message: &Message);
}

// ActorPoll trait inspired by https://docs.rs/stepper
pub trait ActorPoll {
    type Error;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>>;
}

pub trait ActorSense {
    type Message;
    type Error;

    fn sense(&mut self) -> Result<Option<Self::Message>, Self::Error>;
}
