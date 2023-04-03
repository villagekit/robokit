use alloc::boxed::Box;
use core::task::Poll;
use defmt::Format;
use heapless::Vec;

use crate::{
    actuators::{axis::AxisAction, led::LedAction, spindle::SpindleAction, Actuator},
    error::Error,
    runner::{ActuatorSet, Command, Runner, RunnerAction},
};

#[derive(Clone, Copy, Debug, Format)]
pub enum MachineState {
    Idle,
    Start,
    StartLoop,
    Run { command_index: usize },
    RunLoop { command_index: usize },
    Stop,
    StopLoop,
}

pub struct Machine<Leds, const LED_TIMER_HZ: u32, Axes, Spindles>
where
    Leds: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    Axes: ActuatorSet<Action = AxisAction>,
    Spindles: ActuatorSet<Action = SpindleAction>,
{
    runner: Runner<Leds, LED_TIMER_HZ, Axes, Spindles>,
    state: MachineState,
    run_commands: Vec<Command<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>, 32>,
    start_commands: Vec<Command<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>, 4>,
    stop_commands: Vec<Command<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>, 4>,
}

impl<Leds, const LED_TIMER_HZ: u32, Axes, Spindles> Machine<Leds, LED_TIMER_HZ, Axes, Spindles>
where
    Leds: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    Axes: ActuatorSet<Action = AxisAction>,
    Spindles: ActuatorSet<Action = SpindleAction>,
{
    pub fn new(
        runner: Runner<Leds, LED_TIMER_HZ, Axes, Spindles>,
        run_commands: Vec<Command<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>, 32>,
        start_commands: Vec<Command<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>, 4>,
        stop_commands: Vec<Command<Leds::Id, LED_TIMER_HZ, Axes::Id, Spindles::Id>, 4>,
    ) -> Self {
        Self {
            runner,
            state: MachineState::Idle,
            run_commands,
            start_commands,
            stop_commands,
        }
    }

    pub fn start(&mut self) {
        self.state = MachineState::Start;
    }

    pub fn stop(&mut self) {
        self.state = MachineState::Stop;
    }

    pub fn toggle(&mut self) {
        self.state = match self.state {
            MachineState::Idle => MachineState::Start,
            MachineState::Start => MachineState::Stop,
            MachineState::StartLoop => MachineState::Stop,
            MachineState::Run { .. } => MachineState::Stop,
            MachineState::RunLoop { .. } => MachineState::Stop,
            MachineState::Stop => MachineState::Start,
            MachineState::StopLoop => MachineState::Start,
        };
    }

    pub fn poll(&mut self) -> Poll<Result<(), Box<dyn Error>>> {
        match self.state {
            MachineState::Idle => Poll::Ready(Ok(())),
            MachineState::Start => {
                self.runner.run(&RunnerAction::Reset);

                for command in self.start_commands.iter() {
                    defmt::println!("Start: {}", command);

                    self.runner.run(&RunnerAction::Run(*command))
                }

                self.state = MachineState::StartLoop;

                Poll::Pending
            }
            MachineState::StartLoop => {
                if let Poll::Ready(Ok(())) = self.runner.poll() {
                    self.state = MachineState::Run { command_index: 0 };
                }

                Poll::Pending
            }
            MachineState::Run { command_index } => {
                let command = self
                    .run_commands
                    .get(command_index)
                    .expect("Unexpected run command index");

                defmt::println!("Run: {}", command);

                self.runner.run(&RunnerAction::Run(*command));

                self.state = MachineState::RunLoop { command_index };

                Poll::Pending
            }
            MachineState::RunLoop { command_index } => match self.runner.poll() {
                Poll::Ready(Ok(())) => {
                    let next_command_index = command_index + 1;

                    if next_command_index >= self.run_commands.len() {
                        self.state = MachineState::Stop;
                    } else {
                        self.state = MachineState::Run {
                            command_index: next_command_index,
                        };
                    }

                    Poll::Pending
                }
                Poll::Ready(Err(err)) => Poll::Ready(Err(err.into())),
                Poll::Pending => Poll::Pending,
            },
            MachineState::Stop => {
                self.runner.run(&RunnerAction::Reset);

                for command in self.stop_commands.iter() {
                    defmt::println!("Stop: {}", command);

                    self.runner.run(&RunnerAction::Run(*command));
                }

                self.state = MachineState::StopLoop;

                Poll::Pending
            }
            MachineState::StopLoop => {
                if let Poll::Ready(Ok(())) = self.runner.poll() {
                    self.state = MachineState::Idle
                }

                Poll::Pending
            }
        }
    }
}
