use embedded_hal::digital::v2::InputPin;

use crate::actor::ActorSense;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SwitchStatus {
    On,
    Off,
}

#[derive(Debug, Copy, Clone)]
pub struct SwitchUpdate {
    pub status: SwitchStatus,
}

pub struct Switch<P>
where
    P: InputPin,
{
    pin: P,
    current_status: Option<SwitchStatus>,
}

impl<P> Switch<P>
where
    P: InputPin,
{
    pub fn new(pin: P) -> Self {
        Switch {
            pin,
            current_status: None,
        }
    }
}

#[derive(Debug)]
pub enum SwitchError<PinError> {
    Pin(PinError),
}

impl<P> ActorSense for Switch<P>
where
    P: InputPin,
{
    type Message = SwitchUpdate;
    type Error = SwitchError<P::Error>;

    fn sense(&mut self) -> Result<Option<SwitchUpdate>, Self::Error> {
        let is_high = self.pin.is_high().map_err(|err| SwitchError::Pin(err))?;

        let status = if is_high {
            SwitchStatus::On
        } else {
            SwitchStatus::Off
        };

        if Some(status) != self.current_status {
            Ok(Some(SwitchUpdate { status }))
        } else {
            Ok(None)
        }
    }
}
