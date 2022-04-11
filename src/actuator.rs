use embedded_hal::{digital::v2::OutputPin, timer::CountDown};
use fugit::HertzU32 as Hertz;
use nb;
use void;

pub trait Command<Message> {
    type Error;

    fn command(&mut self, message: Message) -> ();
    fn wait(&mut self) -> nb::Result<(), Self::Error>;
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

pub type LedError = void::Void;

impl<P, T> Command<LedBlink> for Led<P, T>
where
    P: OutputPin,
    T: CountDown<Time = Hertz>,
{
    fn command(&mut self, message: LedBlink) -> () {
        self.timer.start(message.duration);
    }

    type Error = LedError;

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        self.pin.set_high().ok();

        self.timer.wait()
    }
}
