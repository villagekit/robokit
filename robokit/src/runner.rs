use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use heapless::{Deque, FnvIndexMap};

use crate::actuators::BoxActuator;
use crate::actuators::{axis::AxisAction, led::LedAction, spindle::SpindleAction, Actuator};
use crate::error::BoxError;

#[derive(Clone, Copy, Debug, Format)]
pub enum Command<'a, const LED_TIMER_HZ: u32> {
    Led(&'a str, LedAction<LED_TIMER_HZ>),
    Axis(&'a str, AxisAction),
    Spindle(&'a str, SpindleAction),
}

#[derive(Clone, Copy, Debug, Format)]
pub enum RunnerAction<Command> {
    Run(Command),
    Reset,
}

pub struct Runner<'a, const LED_TIMER_HZ: u32> {
    active_commands: Deque<Command<'a, LED_TIMER_HZ>, 8>,
    leds: FnvIndexMap<&'a str, BoxActuator<LedAction<LED_TIMER_HZ>>, 16>,
    axes: FnvIndexMap<&'a str, BoxActuator<AxisAction>, 16>,
    spindles: FnvIndexMap<&'a str, BoxActuator<SpindleAction>, 16>,
}

impl<'a, const LED_TIMER_HZ: u32> Runner<'a, LED_TIMER_HZ> {
    pub fn new(
        leds: FnvIndexMap<&'a str, BoxActuator<LedAction<LED_TIMER_HZ>>, 16>,
        axes: FnvIndexMap<&'a str, BoxActuator<AxisAction>, 16>,
        spindles: FnvIndexMap<&'a str, BoxActuator<SpindleAction>, 16>,
    ) -> Self {
        Self {
            active_commands: Deque::new(),
            leds,
            axes,
            spindles,
        }
    }
}

/*
#[derive(Debug)]
pub enum RunnerError<'a> {
    NotFound(&'a str),
    Poll(&'a str, Box<dyn Error>),
}
*/

impl<'a, const LED_TIMER_HZ: u32> Actuator for Runner<'a, LED_TIMER_HZ> {
    type Action = RunnerAction<Command<'a, LED_TIMER_HZ>>;
    type Error = BoxError;

    fn run(&mut self, action: &RunnerAction<Command<'a, LED_TIMER_HZ>>) {
        match action {
            RunnerAction::Run(command) => {
                match command {
                    Command::Led(id, action) => {
                        self.leds.get_mut(id).expect("Led not found!").run(action)
                    }
                    Command::Axis(id, action) => {
                        self.axes.get_mut(id).expect("Axis not found!").run(action)
                    }
                    Command::Spindle(id, action) => self
                        .spindles
                        .get_mut(id)
                        .expect("Spindle not found!")
                        .run(action),
                }

                self.active_commands.push_back(*command).unwrap();
            }
            RunnerAction::Reset => self.active_commands.clear(),
        }
    }

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        let num_commands = self.active_commands.len();
        for _command_index in 0..num_commands {
            let command = self.active_commands.pop_front().unwrap();
            let result = match command {
                Command::Led(id, _) => self.leds.get_mut(id).expect("Led not found!").poll(),
                Command::Axis(id, _) => self.axes.get_mut(id).expect("Axis not found!").poll(),
                Command::Spindle(id, _) => self
                    .spindles
                    .get_mut(id)
                    .expect("Spindle not found!")
                    .poll(),
            };

            match result {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(err)) => {
                    self.active_commands.push_back(command).unwrap();

                    return Poll::Ready(Err(err.into()));
                }
                Poll::Pending => {
                    self.active_commands.push_back(command).unwrap();
                }
            }
        }

        if self.active_commands.len() == 0 {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}
