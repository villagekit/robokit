use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use fixed_map::{key::Key, Map};
use heapless::Deque;

use crate::actuators::BoxActuator;
use crate::actuators::{axis::AxisAction, led::LedAction, spindle::SpindleAction, Actuator};
use crate::error::BoxError;

#[derive(Clone, Copy, Debug, Format)]
pub enum Command<const LED_TIMER_HZ: u32, LedId, AxisId, SpindleId>
where
    LedId: Debug + Format,
    AxisId: Debug + Format,
    SpindleId: Debug + Format,
{
    Led(LedId, LedAction<LED_TIMER_HZ>),
    Axis(AxisId, AxisAction),
    Spindle(SpindleId, SpindleAction),
}

#[derive(Clone, Copy, Debug, Format)]
pub enum RunnerAction<Command> {
    Run(Command),
    Reset,
}

pub struct Runner<
    const LED_TIMER_HZ: u32,
    const ACTIVE_COMMMANDS_COUNT: usize,
    LedId,
    AxisId,
    SpindleId,
> where
    LedId: Key + Debug + Format,
    AxisId: Key + Debug + Format,
    SpindleId: Key + Debug + Format,
{
    active_commands: Deque<Command<LED_TIMER_HZ, LedId, AxisId, SpindleId>, ACTIVE_COMMMANDS_COUNT>,
    leds: Map<LedId, BoxActuator<LedAction<LED_TIMER_HZ>>>,
    axes: Map<AxisId, BoxActuator<AxisAction>>,
    spindles: Map<SpindleId, BoxActuator<SpindleAction>>,
}

impl<const LED_TIMER_HZ: u32, const ACTIVE_COMMMANDS_COUNT: usize, LedId, AxisId, SpindleId>
    Runner<LED_TIMER_HZ, ACTIVE_COMMMANDS_COUNT, LedId, AxisId, SpindleId>
where
    LedId: Key + Debug + Format,
    AxisId: Key + Debug + Format,
    SpindleId: Key + Debug + Format,
{
    pub fn new(
        leds: Map<LedId, BoxActuator<LedAction<LED_TIMER_HZ>>>,
        axes: Map<AxisId, BoxActuator<AxisAction>>,
        spindles: Map<SpindleId, BoxActuator<SpindleAction>>,
    ) -> Self {
        Self {
            active_commands: Deque::new(),
            leds,
            axes,
            spindles,
        }
    }
}

impl<const LED_TIMER_HZ: u32, const ACTIVE_COMMMANDS_COUNT: usize, LedId, AxisId, SpindleId>
    Actuator for Runner<LED_TIMER_HZ, ACTIVE_COMMMANDS_COUNT, LedId, AxisId, SpindleId>
where
    LedId: Key + Debug + Format,
    AxisId: Key + Debug + Format,
    SpindleId: Key + Debug + Format,
{
    type Action = RunnerAction<Command<LED_TIMER_HZ, LedId, AxisId, SpindleId>>;
    type Error = BoxError;

    fn run(&mut self, action: &RunnerAction<Command<LED_TIMER_HZ, LedId, AxisId, SpindleId>>) {
        match action {
            RunnerAction::Run(command) => {
                match command {
                    Command::Led(id, action) => {
                        self.leds.get_mut(*id).expect("Led not found!").run(action)
                    }
                    Command::Axis(id, action) => {
                        self.axes.get_mut(*id).expect("Axis not found!").run(action)
                    }
                    Command::Spindle(id, action) => self
                        .spindles
                        .get_mut(*id)
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
