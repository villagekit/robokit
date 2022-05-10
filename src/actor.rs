use core::fmt::Debug;
use core::task::Poll;
use heapless::spsc::{Consumer, Producer, Queue};

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

pub trait ActorSense {
    type Error;

    fn sense(&mut self) -> Result<(), Self::Error>;
}

pub trait ActorPost {
    type Message;

    fn post(&mut self, message: Self::Message);
}

pub struct ActorOutbox<Message>(Queue<Message, 8>);
pub struct ActorOutboxProducer<'a, Message>(Producer<'a, Message, 8>);
pub struct ActorOutboxConsumer<'a, Message>(Consumer<'a, Message, 8>);

impl<Message> ActorOutbox<Message> {
    pub fn new() -> Self {
        Self(Queue::new())
    }

    pub fn split<'a>(
        &'a mut self,
    ) -> (
        ActorOutboxProducer<'a, Message>,
        ActorOutboxConsumer<'a, Message>,
    ) {
        let (producer, consumer) = self.0.split();
        (ActorOutboxProducer(producer), ActorOutboxConsumer(consumer))
    }
}

impl<'a, Message: Debug> ActorOutboxConsumer<'a, Message> {
    fn read(&mut self) -> Option<Message> {
        self.0.dequeue()
    }
}

impl<'a, Message: Debug> ActorPost for ActorOutboxProducer<'a, Message> {
    type Message = Message;

    fn post(&mut self, message: Self::Message) {
        self.0.enqueue(message).unwrap();
    }
}
