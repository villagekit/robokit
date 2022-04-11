use core::task::Poll;

use stm32f7xx_hal::gpio::{Output, Pin};

pub trait Future {
    type Context;
    type Error;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>>;
}

pub trait Command<Message, Future> {
    fn command(&mut self, message: Message) -> Future;
}

pub trait Listen<Event> {
    fn listen(&mut self, event: Event);
}

pub struct Led<P>
where
    P: Pin,
{
    pin: P<Output>,
    delay: Delay
}

pub struct LedBlink {
    duration: u32,
}

pub struct LedBlinkFuture {
    blink: LedBlink
}

pub struct LedError {}

impl Future for LedBlinkFuture {
    type Error = LedError;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        if (self.blink.duration)
    }
}

impl<Pin> Command<LedBlink,  for Led<Pin> {}
