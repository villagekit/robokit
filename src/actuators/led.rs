use core::task::Poll;

use embedded_hal::digital::v2::OutputPin;
use fugit::MillisDurationU32 as MillisDuration;
use fugit_timer::Timer;

use crate::actuator::{Activity, ActivityError, Actuator};

pub struct Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pin: P,
    timer: T,
}

impl<P, T> Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pub fn new(pin: P, timer: T) -> Self {
        Led { pin, timer }
    }
}

pub struct LedBlinkAction {
    pub duration: MillisDuration,
}

impl<'a, P, T> Actuator<LedBlinkAction, LedBlinkActivity<'a, P, T>> for Led<P, T>
where
    P: 'a + OutputPin,
    T: 'a + Timer<1_000>,
{
    fn act(&mut self, action: &LedBlinkAction) -> LedBlinkActivity<'a, P, T> {
        LedBlinkActivity {
            pin: &mut self.pin,
            timer: &mut self.timer,
            status: LedBlinkStatus::Start,
            duration: action.duration,
        }
    }
}

pub enum LedBlinkStatus {
    Start,
    Wait,
    Done,
}

pub struct LedBlinkActivity<'a, P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    pin: &'a P,
    timer: &'a T,
    status: LedBlinkStatus,
    duration: MillisDuration,
}

impl<'a, P, T> Activity for LedBlinkActivity<'a, P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    fn poll(&mut self) -> Poll<Result<(), ActivityError>> {
        // TODO handle errors
        match self.status {
            LedBlinkStatus::Start => {
                self.timer.start(self.duration).unwrap();
                self.pin.set_high().ok();
                self.status = LedBlinkStatus::Wait;

                Poll::Pending
            }
            LedBlinkStatus::Wait => match self.timer.wait() {
                Err(nb::Error::Other(_err)) => {
                    panic!("Unexpected timer.wait() error");
                }
                Err(nb::Error::WouldBlock) => Poll::Pending,
                Ok(()) => {
                    self.status = LedBlinkStatus::Done;
                    Poll::Pending
                }
            },
            LedBlinkStatus::Done => {
                // if the timer isn't cancelled, it's periodic
                // and will automatically return on next call.
                self.timer.cancel().unwrap();

                self.pin.set_low().ok();

                Poll::Ready(Ok(()))
            }
        }
    }
}
