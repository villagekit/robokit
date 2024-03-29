// inspired by https://github.com/rubberduck203/switch-hal

use core::fmt::Debug;
use core::marker::PhantomData;
use defmt::Format;
use embedded_hal::digital::v2::InputPin;
use fugit::{MillisDurationU32 as MillisDuration, TimerDurationU32 as TimerDuration};
use fugit_timer::Timer;
use nb;

use super::Sensor;

#[derive(Copy, Clone, Debug, Format, PartialEq)]
pub enum SwitchStatus {
    On,
    Off,
}

#[derive(Copy, Clone, Debug, Format)]
pub struct SwitchUpdate {
    pub status: SwitchStatus,
}

pub struct SwitchActiveLow;
pub struct SwitchActiveHigh;

#[derive(Copy, Clone, Debug, Format)]
pub struct SwitchDevice<Pin, ActiveLevel, Tim, const TIMER_HZ: u32>
where
    Pin: InputPin,
    Tim: Timer<TIMER_HZ>,
{
    pin: Pin,
    timer: Tim,
    current_status: Option<SwitchStatus>,
    active_level: PhantomData<ActiveLevel>,
    is_debouncing: bool,
}

impl<Pin, ActiveLevel, Tim, const TIMER_HZ: u32> SwitchDevice<Pin, ActiveLevel, Tim, TIMER_HZ>
where
    Pin: InputPin,
    Tim: Timer<TIMER_HZ>,
{
    pub fn new(pin: Pin, timer: Tim) -> Self {
        Self {
            pin,
            timer,
            current_status: None,
            active_level: PhantomData::<ActiveLevel>,
            is_debouncing: false,
        }
    }
}

impl<Pin, Tim, const TIMER_HZ: u32> SwitchDevice<Pin, SwitchActiveHigh, Tim, TIMER_HZ>
where
    Pin: InputPin,
    Tim: Timer<TIMER_HZ>,
{
    pub fn new_active_high(pin: Pin, timer: Tim) -> Self {
        SwitchDevice::<Pin, SwitchActiveHigh, Tim, TIMER_HZ>::new(pin, timer)
    }
}

impl<Pin, Tim, const TIMER_HZ: u32> SwitchDevice<Pin, SwitchActiveLow, Tim, TIMER_HZ>
where
    Pin: InputPin,
    Tim: Timer<TIMER_HZ>,
{
    pub fn new_active_low(pin: Pin, timer: Tim) -> Self {
        SwitchDevice::<Pin, SwitchActiveLow, Tim, TIMER_HZ>::new(pin, timer)
    }
}

pub trait InputSwitch {
    type Error: Debug;

    fn is_active(&self) -> Result<bool, Self::Error>;
}

impl<Pin, Tim, const TIMER_HZ: u32> InputSwitch
    for SwitchDevice<Pin, SwitchActiveLow, Tim, TIMER_HZ>
where
    Pin: InputPin,
    Pin::Error: Debug,
    Tim: Timer<TIMER_HZ>,
{
    type Error = <Pin as InputPin>::Error;

    fn is_active(&self) -> Result<bool, Self::Error> {
        self.pin.is_low()
    }
}

impl<Pin, Tim, const TIMER_HZ: u32> InputSwitch
    for SwitchDevice<Pin, SwitchActiveHigh, Tim, TIMER_HZ>
where
    Pin: InputPin,
    Pin::Error: Debug,
    Tim: Timer<TIMER_HZ>,
{
    type Error = <Pin as InputPin>::Error;

    fn is_active(&self) -> Result<bool, Self::Error> {
        self.pin.is_high()
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub enum SwitchError<PinError: Debug, TimerError: Debug> {
    PinRead(PinError),
    TimerStart(TimerError),
    TimerWait(TimerError),
    TimerCancel(TimerError),
}

impl<Pin, ActiveLevel, Tim, const TIMER_HZ: u32> Sensor
    for SwitchDevice<Pin, ActiveLevel, Tim, TIMER_HZ>
where
    Self: InputSwitch,
    Pin: InputPin,
    Tim: Timer<TIMER_HZ>,
{
    type Message = SwitchUpdate;
    type Error = SwitchError<<Self as InputSwitch>::Error, Tim::Error>;

    fn sense(&mut self) -> Result<Option<SwitchUpdate>, Self::Error> {
        if self.is_debouncing {
            match self.timer.wait() {
                Ok(()) => {
                    self.timer.cancel().map_err(SwitchError::TimerCancel)?;
                    self.is_debouncing = false;
                }
                Err(nb::Error::WouldBlock) => return Ok(None),
                Err(nb::Error::Other(err)) => return Err(SwitchError::TimerWait(err)),
            }
        }

        let is_active = self.is_active().map_err(SwitchError::PinRead)?;

        let status = match is_active {
            true => SwitchStatus::On,
            false => SwitchStatus::Off,
        };

        if Some(status) != self.current_status {
            self.current_status = Some(status);

            let debounce_duration: TimerDuration<TIMER_HZ> =
                MillisDuration::from_ticks(2).convert();
            self.is_debouncing = true;
            self.timer
                .start(debounce_duration)
                .map_err(SwitchError::TimerStart)?;

            Ok(Some(SwitchUpdate { status }))
        } else {
            Ok(None)
        }
    }
}
