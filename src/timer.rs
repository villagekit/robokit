use embedded_hal::timer::CountDown;
use embedded_time::{duration::Nanoseconds, fixed_point::FixedPoint};
use fugit::NanosDurationU32;
use fugit_timer::Timer as FugitTimer;
use nb;
use void::Void;

/// Wrapper around a timer for stepper which needs
/// CountDown::Timer = embedded_time::Nanoseconds
pub struct EmbeddedTimeCounter<T>(pub T);

impl<Timer> CountDown for EmbeddedTimeCounter<Timer>
where
    Timer: FugitTimer<1_000_000_000>,
{
    type Time = Nanoseconds;

    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Self::Time>,
    {
        self.0
            .start(NanosDurationU32::from_ticks(timeout.into().integer()))
            .unwrap()
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        match self.0.wait() {
            Ok(()) => Ok(()),
            Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
            Err(nb::Error::Other(_)) => {
                unreachable!("Caught error from infallible method")
            }
        }
    }
}
