// inspired by https://github.com/rubberduck203/switch-hal

use core::marker::PhantomData;
use defmt::Format;
use embedded_hal::digital::v2::InputPin;

use crate::actor::ActorSense;

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
pub struct Switch<Pin, ActiveLevel>
where
    Pin: InputPin,
{
    pin: Pin,
    current_status: Option<SwitchStatus>,
    active_level: PhantomData<ActiveLevel>,
}

impl<Pin, ActiveLevel> Switch<Pin, ActiveLevel>
where
    Pin: InputPin,
{
    pub fn new(pin: Pin) -> Self {
        Switch {
            pin,
            current_status: None,
            active_level: PhantomData::<ActiveLevel>,
        }
    }
}

pub trait InputSwitch {
    type Error;

    fn is_active(&self) -> Result<bool, Self::Error>;
}

impl<Pin> InputSwitch for Switch<Pin, SwitchActiveLow>
where
    Pin: InputPin,
{
    type Error = <Pin as InputPin>::Error;

    fn is_active(&self) -> Result<bool, Self::Error> {
        self.pin.is_low()
    }
}

impl<Pin> InputSwitch for Switch<Pin, SwitchActiveHigh>
where
    Pin: InputPin,
{
    type Error = <Pin as InputPin>::Error;

    fn is_active(&self) -> Result<bool, Self::Error> {
        self.pin.is_high()
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub enum SwitchError<PinError> {
    Pin(PinError),
}

impl<Pin, ActiveLevel> ActorSense for Switch<Pin, ActiveLevel>
where
    Self: InputSwitch,
    Pin: InputPin,
{
    type Message = SwitchUpdate;
    type Error = SwitchError<<Self as InputSwitch>::Error>;

    fn sense(&mut self) -> Result<Option<SwitchUpdate>, Self::Error> {
        let is_active = self.is_active().map_err(|err| SwitchError::Pin(err))?;

        let status = if is_active {
            SwitchStatus::On
        } else {
            SwitchStatus::Off
        };

        if Some(status) != self.current_status {
            self.current_status = Some(status);

            Ok(Some(SwitchUpdate { status }))
        } else {
            Ok(None)
        }
    }
}
