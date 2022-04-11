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

pub enum LedStatus {
    Start,
    Wait,
    Done,
}

pub struct Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pin: P,
    timer: T,
    status: LedStatus,
    duration: Option<MillisDuration>,
}

impl<P, T> Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pub fn new(pin: P, timer: T) -> Self {
        Led {
            pin,
            timer,
            status: LedStatus::Start,
            duration: None,
        }
    }
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
        self.status = LedStatus::Start;
        self.duration = Some(message.duration);
    }
}

impl<P, T> Waitable for Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    type Error = Void;

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        // TODO handle errors
        match self.status {
            LedStatus::Start => {
                defmt::println!("START!");

                self.timer.start(self.duration.unwrap()).unwrap();
                self.pin.set_high().ok();
                self.status = LedStatus::Wait;

                Err(nb::Error::WouldBlock)
            }
            LedStatus::Wait => match self.timer.wait() {
                Err(nb::Error::Other(_err)) => {
                    panic!("Unexpected timer.wait() error");
                }
                Err(nb::Error::WouldBlock) => Err(nb::Error::WouldBlock),
                Ok(()) => {
                    self.status = LedStatus::Done;
                    Err(nb::Error::WouldBlock)
                }
            },
            LedStatus::Done => {
                defmt::println!("DONE!");

                self.pin.set_low().ok();

                Ok(())
            }
        }
    }
}
