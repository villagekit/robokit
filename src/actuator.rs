use core::future::Future;

use lilos::exec::sleep_for;

use stm32f7xx_hal::gpio::{Output, Pin, PushPull};

pub trait Command<Message> {
    fn command(&mut self, message: Message) -> Future<()>;
}

pub trait Listen<Event> {
    fn listen(&mut self, event: Event);
}

pub struct Led {
    pin: Pin<Output<PushPull>>,
}

pub struct LedBlink {
    duration: u32,
}

impl Command<LedBlink> for Led {
    async fn command(&mut self, message: LedBlink) -> Future<()> {
        self.pin.set_high();
        sleep_for(message.duration).await;
        self.pin.set_low();
    }
}
