use core::task::Poll;

pub trait ActorReceive {
    type Message;

    fn receive(&mut self, message: &Self::Message);
}

pub trait ActorPoll {
    type Error;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>>;
}
