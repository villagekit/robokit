use embedded_hal::{digital::v2::OutputPin, timer::CountDown};
use fugit::HertzU32 as Hertz;
use nb;
use void;

pub trait Waitable {
    type Error;

    fn wait(&mut self) -> nb::Result<(), Self::Error>;
}

pub trait Command<Message, Future> {
    fn command(&mut self, message: Message) -> Future;
}

pub trait Listen<Event> {
    fn listen(&mut self, event: Event);
}

pub struct Led<P, T>
where
    P: OutputPin,
    T: CountDown,
{
    pin: P,
    timer: T,
}

pub struct LedBlink {
    duration: Hertz,
}

pub struct LedBlinkFuture<P, T>
where
    P: OutputPin,
    T: CountDown,
{
    pin: P,
    timer: T,
}

pub struct LedError {}

impl<P, T> Waitable for LedBlinkFuture<P, T>
where
    P: OutputPin,
    T: CountDown,
{
    type Error = void::Void;

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        self.pin.set_high().ok();

        self.timer.wait()
    }
}

impl<P, T> Command<LedBlink, LedBlinkFuture<P, T>> for Led<P, T>
where
    P: OutputPin,
    T: CountDown<Time = Hertz>,
{
    fn command(&mut self, message: LedBlink) -> LedBlinkFuture<P, T> {
        self.timer.start(message.duration);

        LedBlinkFuture {
            pin: self.pin,
            timer: self.timer,
        }
    }
}
