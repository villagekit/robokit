use crate::error::Error;

pub trait Actuator<'a> {
    type Message;
    type Future: Future + 'a;

    fn command(&'a mut self, message: &Self::Message) -> Self::Future;
}

pub trait Future {
    fn poll(&mut self) -> core::task::Poll<Result<(), Error>>;
}

pub trait Listen<Event> {
    fn listen(&mut self, event: Event);
}
