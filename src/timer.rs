use core::convert::Infallible;
use core::ops;
use fugit::TimerDuration;
use fugit_timer::Timer as FugitTimer;
use nb;
use stepper::embedded_hal::timer::nb::CountDown;
use stepper::embedded_time::{duration::*, ConversionError};

pub struct StepperTimer<Timer, const FREQ: u32>(pub Timer);

impl<Timer, const FREQ: u32> CountDown for StepperTimer<Timer, FREQ>
where
    Timer: FugitTimer<FREQ>,
{
    type Error = Infallible;

    type Time = StepperTicks<FREQ>;

    fn start<Ticks>(&mut self, ticks: Ticks) -> Result<(), Self::Error>
    where
        Ticks: Into<Self::Time>,
    {
        /*
        // subtract time spent between now and previous timer
        let sofar_ticks = self.0.now().ticks();
        let wait_ticks = ticks.into().0.ticks();
        let mut ticks = if wait_ticks <= sofar_ticks {
            0
        } else {
            wait_ticks - sofar_ticks
        };
        if ticks > 50 {
            defmt::println!("timer: {} - {} = {}", wait_ticks, sofar_ticks, ticks);
        }
        */

        let mut ticks = ticks.into().0.ticks();

        // wait to discard any interrupt events that triggered before we started.
        self.0.wait().ok();

        // if below minimum, set to minimum: 2 ticks
        if ticks < 2 {
            ticks = 2;
        }

        self.0
            .start(TimerDuration::<u32, FREQ>::from_ticks(ticks))
            .unwrap();
        Ok(())
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        match self.0.wait() {
            Ok(()) => {
                /*
                // start another timer to count time between now and next timer
                self.0
                    .start(TimerDuration::<u32, FREQ>::from_ticks(65535))
                    .unwrap();
                */

                Ok(())
            }
            Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
            Err(nb::Error::Other(_)) => {
                unreachable!("Caught error from infallible method")
            }
        }
    }
}

pub struct StepperTicks<const FREQ: u32>(pub TimerDuration<u32, FREQ>);

macro_rules! impl_embedded_time_conversions {
    ($($duration:ident,)*) => {
        $(
            impl<const FREQ: u32> TryFrom<embedded_time::duration::$duration>
                for StepperTicks<FREQ>
            {
                type Error = ConversionError;

                fn try_from(duration: embedded_time::duration::$duration)
                    -> Result<Self, Self::Error>
                {
                    let ticks = duration.into_ticks::<u32>(Fraction::new(1, FREQ))?;
                    Ok(Self(TimerDuration::<u32, FREQ>::from_ticks(ticks)))
                }
            }
        )*
    };
}

impl_embedded_time_conversions!(
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
);

impl<const FREQ: u32> ops::Sub for StepperTicks<FREQ> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        StepperTicks(self.0 - other.0)
    }
}

impl<const FREQ: u32> defmt::Format for StepperTicks<FREQ> {
    fn format(&self, f: defmt::Formatter) {
        self.0.format(f)
    }
}
