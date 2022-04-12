use crate::actuators::led::LedBlinkMessage;

pub enum Command {
    GreenLed(LedBlinkMessage),
    BlueLed(LedBlinkMessage),
    RedLed(LedBlinkMessage),
}
