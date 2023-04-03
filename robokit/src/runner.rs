use alloc::boxed::Box;
use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use heapless::Deque;

use crate::actuators::{axis::AxisAction, led::LedAction, spindle::SpindleAction, Actuator};
use crate::error::{BoxError, Error};

#[derive(Clone, Copy, Debug, Format)]
pub enum Command<LedId, const LED_TIMER_HZ: u32, AxisId, SpindleId>
where
    LedId: Copy + Clone + Debug + Format,
    AxisId: Copy + Clone + Debug + Format,
    SpindleId: Copy + Clone + Debug + Format,
{
    Led(LedId, LedAction<LED_TIMER_HZ>),
    Axis(AxisId, AxisAction),
    Spindle(SpindleId, SpindleAction),
}

#[derive(Clone, Copy, Debug, Format)]
pub enum RunnerAction<LedId, const LED_TIMER_HZ: u32, AxisId, SpindleId>
where
    LedId: Copy + Clone + Debug + Format,
    AxisId: Copy + Clone + Debug + Format,
    SpindleId: Copy + Clone + Debug + Format,
{
    Run(Command<LedId, LED_TIMER_HZ, AxisId, SpindleId>),
    Reset,
}

pub struct Runner<Leds, const LED_TIMER_HZ: u32, Axes, Spindles>
where
    Leds: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    Axes: ActuatorSet<Action = AxisAction>,
    Spindles: ActuatorSet<Action = SpindleAction>,
{
    active_commands: Deque<Command<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>, 8>,
    leds: Leds,
    axes: Axes,
    spindles: Spindles,
}

impl<Leds, const LED_TIMER_HZ: u32, Axes, Spindles> Runner<Leds, LED_TIMER_HZ, Axes, Spindles>
where
    Leds: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    Axes: ActuatorSet<Action = AxisAction>,
    Spindles: ActuatorSet<Action = SpindleAction>,
{
    pub fn new(leds: Leds, axes: Axes, spindles: Spindles) -> Self {
        Self {
            active_commands: Deque::new(),
            leds,
            axes,
            spindles,
        }
    }
}

impl<Leds, const LED_TIMER_HZ: u32, Axes, Spindles> Actuator
    for Runner<Leds, LED_TIMER_HZ, Axes, Spindles>
where
    Leds: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    Axes: ActuatorSet<Action = AxisAction>,
    Spindles: ActuatorSet<Action = SpindleAction>,
{
    type Action = RunnerAction<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>;
    type Error = BoxError;

    fn run(&mut self, action: &RunnerAction<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>) {
        match action {
            RunnerAction::Run(command) => {
                match command {
                    Command::Led(id, action) => self.leds.run(id, action),
                    Command::Axis(id, action) => self.axes.run(id, action),
                    Command::Spindle(id, action) => self.spindles.run(id, action),
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
                Command::Led(id, _) => self.leds.poll(&id),
                Command::Axis(id, _) => self.axes.poll(&id),
                Command::Spindle(id, _) => self.spindles.poll(&id),
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

pub trait ActuatorSet {
    type Action: Copy + Clone + Debug + Format;
    type Id: Copy + Clone + Debug + Format;

    fn run(&mut self, id: &Self::Id, action: &Self::Action);
    fn poll(&mut self, id: &Self::Id) -> Poll<Result<(), Box<dyn Error>>>;
}
