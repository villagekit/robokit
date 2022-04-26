use core::convert::From;
use core::ops::{Add, Div, Mul, Rem, Sub};
use embedded_hal::timer::CountDown;
use embedded_time::{duration::Nanoseconds, fixed_point::FixedPoint, TimeInt};
use fugit::NanosDurationU32;
use fugit_timer::Timer as FugitTimer;
use nb;
use num::{Integer, Num, One, Zero};
use void::Void;

/// Wrapper around a timer for stepper which needs
/// CountDown::Timer = embedded_time::Nanoseconds
pub struct EmbeddedTimeCounter<Timer, const FREQ: u32>(pub Timer);

impl<Timer, const FREQ: u32> CountDown for EmbeddedTimeCounter<Timer, FREQ>
where
    Timer: FugitTimer<FREQ>,
{
    type Time = EmbeddedTimeCounterTime;

    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Self::Time>,
    {
        self.0
            .start(NanosDurationU32::from_ticks(timeout.into()))
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

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct EmbeddedTimeCounterTime(pub u32);

impl From<Nanoseconds> for EmbeddedTimeCounterTime {
    fn from(time: Nanoseconds) {
        time.integer()
    }
}

impl Add for EmbeddedTimeCounterTime {
    type Output = EmbeddedTimeCounterTime;

    fn add(self, rhs: EmbeddedTimeCounterTime) -> Self::Output {
        self.0 + rhs.0
    }
}

impl Sub for EmbeddedTimeCounterTime {
    type Output = EmbeddedTimeCounterTime;

    fn sub(self, rhs: EmbeddedTimeCounterTime) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Mul for EmbeddedTimeCounterTime {
    type Output = EmbeddedTimeCounterTime;

    fn mul(self, rhs: EmbeddedTimeCounterTime) -> Self::Output {
        self.0 * rhs.0
    }
}

impl Div for EmbeddedTimeCounterTime {
    type Output = EmbeddedTimeCounterTime;

    fn div(self, rhs: EmbeddedTimeCounterTime) -> Self::Output {
        self.0 / rhs.0
    }
}

impl Rem for EmbeddedTimeCounterTime {
    type Output = EmbeddedTimeCounterTime;

    fn rem(self, rhs: EmbeddedTimeCounterTime) -> Self::Output {
        self.0 % rhs.0
    }
}

impl Zero for EmbeddedTimeCounterTime {
    fn zero() -> Self {
        <u32 as Zero>::zero()
    }
}

impl One for EmbeddedTimeCounterTime {
    fn one() -> Self {
        <u32 as One>::one()
    }
}

impl Num for EmbeddedTimeCounterTime {
    type FromStrRadixErr = <u32 as Num>::FromStrRadixErr;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        <u32 as Num>::from_str_radix(str, radix)
    }
}

impl Integer for EmbeddedTimeCounterTime {
    fn div_floor(&self, other: &Self) -> Self {
        self.0.div_floor(other)
    }
    fn mod_floor(&self, other: &Self) -> Self {
        self.0.mod_floor(other)
    }
    fn gcd(&self, other: &Self) -> Self {
        self.0.gcd(other)
    }
    fn lcm(&self, other: &Self) -> Self {
        self.0.lcm(other)
    }
    fn divides(&self, other: &Self) -> bool {
        self.0.divides(other)
    }
    fn is_multiple_of(&self, other: &Self) -> bool {
        self.0.is_multiple_of(other)
    }
    fn is_even(&self) -> bool {
        self.0.is_even()
    }
    fn is_odd(&self) -> bool {
        self.0.is_odd()
    }
    fn div_rem(&self, other: &Self) -> (Self, Self) {
        self.0.div_rem(other)
    }
    fn div_ceil(&self, other: &Self) -> Self {
        self.0.div_ceil(other)
    }
}
