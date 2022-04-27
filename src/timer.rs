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
        let ticks = ticks.into();
        defmt::println!("ticks: {}", ticks);
        self.0.start(ticks.0).unwrap();
        Ok(())
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        match self.0.wait() {
            Ok(()) => Ok(()),
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
                    defmt::println!("freq: {}", FREQ);
                    defmt::println!("duration: {}", duration.integer());
                    let ticks =
                        duration.to_generic::<u32>(Fraction::new(1, FREQ))?;
                    defmt::println!("try ticks: {}", ticks.integer());
                    Ok(Self(TimerDuration::<u32, FREQ>::from_ticks(ticks.integer())))
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
