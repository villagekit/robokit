use embedded_hal::timer::CountDown;
use embedded_time::duration::Nanoseconds;
use fugit_timer::Timer as FugitTimer;
use nb;
use void::Void;

/// Wrapper around a timer for stepper which needs
/// CountDown::Timer = embedded_time::Nanoseconds
pub struct EmbeddedTimeCounter<T>(pub T);

impl<Timer> CountDown for EmbeddedTimeCounter<Timer>
where
    Timer: FugitTimer<1_000_000>,
{
    type Time = Nanoseconds;

    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Self::Time>,
    {
        self.start(timeout.into())
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        self.wait()
    }
}
