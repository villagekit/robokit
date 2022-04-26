use core::convert::From;
use core::fmt;
use core::ops::{Add, Div, Mul, Rem, Sub};
use embedded_hal::timer::CountDown;
use embedded_time::{duration::Nanoseconds, fixed_point::FixedPoint, fraction::Fraction, TimeInt};
use fugit::MicrosDurationU32;
use fugit_timer::Timer as FugitTimer;
use nb;
use num::{
    traits::{WrappingAdd, WrappingSub},
    Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Integer, Num, One, Zero,
};
use void::Void;

/// Wrapper around a timer for stepper which needs
/// CountDown::Timer = embedded_time::Nanoseconds
pub struct EmbeddedTimeCounter<Timer>(pub Timer);

impl<Timer> CountDown for EmbeddedTimeCounter<Timer>
where
    Timer: FugitTimer<1_000_000>,
{
    type Time = EmbeddedTimeCounterTime;

    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Self::Time>,
    {
        self.0
            .start(MicrosDurationU32::from_ticks(timeout.into().0))
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

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct EmbeddedTimeCounterTime(pub u32);

impl From<Nanoseconds> for EmbeddedTimeCounterTime {
    fn from(time: Nanoseconds) -> Self {
        EmbeddedTimeCounterTime(time.integer())
    }
}

impl Add for EmbeddedTimeCounterTime {
    type Output = EmbeddedTimeCounterTime;

    fn add(self, rhs: EmbeddedTimeCounterTime) -> Self::Output {
        EmbeddedTimeCounterTime(self.0 + rhs.0)
    }
}

impl Sub for EmbeddedTimeCounterTime {
    type Output = EmbeddedTimeCounterTime;

    fn sub(self, rhs: EmbeddedTimeCounterTime) -> Self::Output {
        EmbeddedTimeCounterTime(self.0 - rhs.0)
    }
}

impl Mul for EmbeddedTimeCounterTime {
    type Output = EmbeddedTimeCounterTime;

    fn mul(self, rhs: EmbeddedTimeCounterTime) -> Self::Output {
        EmbeddedTimeCounterTime(self.0 * rhs.0)
    }
}

impl Div for EmbeddedTimeCounterTime {
    type Output = EmbeddedTimeCounterTime;

    fn div(self, rhs: EmbeddedTimeCounterTime) -> Self::Output {
        EmbeddedTimeCounterTime(self.0 / rhs.0)
    }
}

impl Rem for EmbeddedTimeCounterTime {
    type Output = EmbeddedTimeCounterTime;

    fn rem(self, rhs: EmbeddedTimeCounterTime) -> Self::Output {
        EmbeddedTimeCounterTime(self.0 % rhs.0)
    }
}

impl Zero for EmbeddedTimeCounterTime {
    fn zero() -> Self {
        EmbeddedTimeCounterTime(<u32 as Zero>::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl One for EmbeddedTimeCounterTime {
    fn one() -> Self {
        EmbeddedTimeCounterTime(<u32 as One>::one())
    }
}

impl Num for EmbeddedTimeCounterTime {
    type FromStrRadixErr = <u32 as Num>::FromStrRadixErr;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        <u32 as Num>::from_str_radix(str, radix).map(|v| EmbeddedTimeCounterTime(v))
    }
}

impl Integer for EmbeddedTimeCounterTime {
    fn div_floor(&self, other: &Self) -> Self {
        EmbeddedTimeCounterTime(Integer::div_floor(&self.0, &other.0))
    }
    fn mod_floor(&self, other: &Self) -> Self {
        EmbeddedTimeCounterTime(self.0.mod_floor(&other.0))
    }
    fn gcd(&self, other: &Self) -> Self {
        EmbeddedTimeCounterTime(self.0.gcd(&other.0))
    }
    fn lcm(&self, other: &Self) -> Self {
        EmbeddedTimeCounterTime(self.0.lcm(&other.0))
    }
    fn divides(&self, other: &Self) -> bool {
        self.0.divides(&other.0)
    }
    fn is_multiple_of(&self, other: &Self) -> bool {
        self.0.is_multiple_of(&other.0)
    }
    fn is_even(&self) -> bool {
        self.0.is_even()
    }
    fn is_odd(&self) -> bool {
        self.0.is_odd()
    }
    fn div_rem(&self, other: &Self) -> (Self, Self) {
        let (a, b) = self.0.div_rem(&other.0);
        (EmbeddedTimeCounterTime(a), EmbeddedTimeCounterTime(b))
    }
}

impl Bounded for EmbeddedTimeCounterTime {
    fn min_value() -> Self {
        EmbeddedTimeCounterTime(<u32 as Bounded>::min_value())
    }
    fn max_value() -> Self {
        EmbeddedTimeCounterTime(<u32 as Bounded>::max_value())
    }
}

impl WrappingAdd for EmbeddedTimeCounterTime {
    fn wrapping_add(&self, v: &Self) -> Self {
        EmbeddedTimeCounterTime(self.0.wrapping_add(v.0))
    }
}

impl WrappingSub for EmbeddedTimeCounterTime {
    fn wrapping_sub(&self, v: &Self) -> Self {
        EmbeddedTimeCounterTime(self.0.wrapping_sub(v.0))
    }
}

impl CheckedAdd for EmbeddedTimeCounterTime {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        self.0.checked_add(v.0).map(|v| EmbeddedTimeCounterTime(v))
    }
}

impl CheckedSub for EmbeddedTimeCounterTime {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        self.0.checked_sub(v.0).map(|v| EmbeddedTimeCounterTime(v))
    }
}

impl CheckedMul for EmbeddedTimeCounterTime {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        self.0.checked_mul(v.0).map(|v| EmbeddedTimeCounterTime(v))
    }
}

impl CheckedDiv for EmbeddedTimeCounterTime {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        self.0.checked_div(v.0).map(|v| EmbeddedTimeCounterTime(v))
    }
}

impl From<u32> for EmbeddedTimeCounterTime {
    fn from(time: u32) -> Self {
        EmbeddedTimeCounterTime(time)
    }
}

impl Mul<Fraction> for EmbeddedTimeCounterTime {
    type Output = EmbeddedTimeCounterTime;

    fn mul(self, rhs: Fraction) -> Self::Output {
        EmbeddedTimeCounterTime(self.0 * rhs)
    }
}

impl Div<Fraction> for EmbeddedTimeCounterTime {
    type Output = EmbeddedTimeCounterTime;

    fn div(self, rhs: Fraction) -> Self::Output {
        EmbeddedTimeCounterTime(self.0 * rhs)
    }
}

impl fmt::Display for EmbeddedTimeCounterTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TimeInt for EmbeddedTimeCounterTime {}
