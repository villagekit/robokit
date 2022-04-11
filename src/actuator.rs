use embedded_hal::{digital::v2::OutputPin, timer::CountDown};
use fugit::MillisDurationU32 as MillisDuration;
use nb;
use void;

pub trait Command<Message> {
    fn command(&mut self, message: Message) -> ();
}

pub trait Waitable {
    type Error;

    fn wait(&mut self) -> nb::Result<(), Self::Error>;
}

pub trait Listen<Event> {
    fn listen(&mut self, event: Event);
}

pub struct Led<P, T>
where
    P: OutputPin,
    T: CountDown<Time = MillisDuration>,
{
    pub pin: P,
    pub timer: T,
}

pub struct LedBlink {
    pub duration: MillisDuration,
}

impl<P, T> Command<LedBlink> for Led<P, T>
where
    P: OutputPin,
    T: CountDown<Time = MillisDuration>,
{
    fn command(&mut self, message: LedBlink) -> () {
        self.pin.set_high().ok();
        self.timer.start(message.duration);
    }
}

impl<P, T> Waitable for Led<P, T>
where
    P: OutputPin,
    T: CountDown<Time = MillisDuration>,
{
    type Error = void::Void;

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        match self.timer.wait() {
            Err(err) => Err(err),
            Ok(()) => {
                self.pin.set_low().ok();

                Ok(())
            }
        }
    }
}
