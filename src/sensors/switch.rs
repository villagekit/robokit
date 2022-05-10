use embedded_hal::digital::v2::InputPin;

use crate::actor::{ActorPost, ActorSense};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SwitchStatus {
    On,
    Off,
}

#[derive(Debug, Copy, Clone)]
pub struct SwitchUpdate {
    pub status: SwitchStatus,
}

pub struct Switch<P, O>
where
    P: InputPin,
    O: ActorPost<Message = SwitchUpdate>,
{
    pin: P,
    outbox: O,
    current_status: Option<SwitchStatus>,
}

impl<P, O> Switch<P, O>
where
    P: InputPin,
    O: ActorPost<Message = SwitchUpdate>,
{
    pub fn new(pin: P, outbox: O) -> Self {
        Switch {
            pin,
            outbox,
            current_status: None,
        }
    }
}

#[derive(Debug)]
pub enum SwitchError<PinError> {
    Pin(PinError),
}

impl<P, O> ActorSense for Switch<P, O>
where
    P: InputPin,
    O: ActorPost<Message = SwitchUpdate>,
{
    type Error = SwitchError<P::Error>;

    fn sense(&mut self) -> Result<(), Self::Error> {
        let is_high = self.pin.is_high().map_err(|err| SwitchError::Pin(err))?;

        let status = if is_high {
            SwitchStatus::On
        } else {
            SwitchStatus::Off
        };

        if Some(status) != self.current_status {
            self.outbox.post(SwitchUpdate { status });
        }

        Ok(())
    }
}
