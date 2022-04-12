use core::task::Poll;

use embedded_hal::digital::v2::OutputPin;
use fugit::MillisDurationU32 as MillisDuration;
use fugit_timer::Timer;

use crate::actuator::{Activity, ActivityError, Actuator};
use crate::util::ref_mut::RefMut;

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

impl<P, T> Actuator for Led<P, T>
where
    P: OutputPin,
    T: Timer<1_000>,
{
    type Action = LedBlinkAction;
    type Output = LedBlinkActivity<RefMut<Led<P, T>>>;

    fn act(&mut self, action: &LedBlinkAction) -> Self::Output {
        LedBlinkActivity {
            led: RefMut(self),
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

pub struct LedBlinkActivity<L> {
    led: L,
    status: LedBlinkStatus,
    duration: MillisDuration,
}

impl<L> Activity for LedBlinkActivity<L> {
    fn poll(mut self) -> Poll<Result<(), ActivityError>> {
        // TODO handle errors
        match self.status {
            LedBlinkStatus::Start => {
                self.led.timer.start(self.duration).unwrap();
                self.led.pin.set_high().ok();
                self.status = LedBlinkStatus::Wait;

                Poll::Pending
            }
            LedBlinkStatus::Wait => match self.led.timer.wait() {
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
                self.led.timer.cancel().unwrap();

                self.led.pin.set_low().ok();

                Poll::Ready(Ok(()))
            }
        }
    }
}
