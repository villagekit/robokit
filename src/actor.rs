use core::task::Poll;

// ActorReceive trait inspired by https://github.com/rtic-rs/rfcs/pull/0052
pub trait ActorReceive {
    type Message;

    fn receive(&mut self, message: &Self::Message);
}

// ActorPoll trait inspired by https://docs.rs/stepper
pub trait ActorPoll {
    type Error;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>>;
}
