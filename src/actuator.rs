use embedded_hal::digital::v2::OutputPin;
use fugit::MillisDurationU32 as MillisDuration;
use fugit_timer::Timer;
use nb;
use void::Void;

pub trait Commandable<Message> {
    fn command(&mut self, message: &Message) -> ();
}

pub trait Waitable {
    type Error: core::fmt::Debug;

    fn wait(&mut self) -> nb::Result<(), Self::Error>;
}

pub trait Listen<Event> {
    fn listen(&mut self, event: Event);
}

pub struct Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pub pin: P,
    pub timer: T,
}

pub struct LedBlink {
    pub duration: MillisDuration,
}

impl<P, T> Commandable<LedBlink> for Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    fn command(&mut self, message: &LedBlink) -> () {
        defmt::println!("HIGH!");

        self.timer.start(message.duration).unwrap();

        // TODO handle error
        self.pin.set_high().ok();
    }
}

impl<P, T> Waitable for Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    type Error = Void;

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        match self.timer.wait() {
            Err(nb::Error::Other(_err)) => {
                panic!("Unexpected fugit error");
            }
            Err(nb::Error::WouldBlock) => Err(nb::Error::WouldBlock),
            Ok(()) => {
                defmt::println!("LOW!");

                // TODO handle error
                self.pin.set_low().ok();

                Ok(())
            }
        }
    }
}
