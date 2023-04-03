use core::task::Poll;
use defmt::Format;
use fugit::ExtU32;
use heapless::Vec;

use crate::{
    actuators::{
        axis::AxisAction,
        led::LedAction,
        spindle::{SpindleAction, SpindleStatus},
        Actuator,
    },
    runner::{AnyRunner, AxisId, Command, LedId, RunnerAction, SpindleId},
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

pub struct Machine<Runner: AnyRunner> {
    runner: Runner,
    state: MachineState,
    run_commands: Vec<Command, 32>,
    start_commands: Vec<Command, 4>,
    stop_commands: Vec<Command, 4>,
}

impl<Runner: AnyRunner> Machine<Runner> {
    pub fn new(runner: Runner) -> Self {
        let run_commands: [Command; 8] = [
            Command::Led(
                LedId::Green,
                LedAction::Blink {
                    duration: 50.millis(),
                },
            ),
            Command::Led(
                LedId::Blue,
                LedAction::Blink {
                    duration: 100.millis(),
                },
            ),
            Command::Led(
                LedId::Red,
                LedAction::Blink {
                    duration: 200.millis(),
                },
            ),
            Command::Axis(
                AxisId::X,
                AxisAction::MoveRelative {
                    max_velocity_in_millimeters_per_sec: 10_f64,
                    distance_in_millimeters: 40_f64,
                },
            ),
            Command::Led(
                LedId::Red,
                LedAction::Blink {
                    duration: 50.millis(),
                },
            ),
            Command::Led(
                LedId::Blue,
                LedAction::Blink {
                    duration: 100.millis(),
                },
            ),
            Command::Led(
                LedId::Green,
                LedAction::Blink {
                    duration: 200.millis(),
                },
            ),
            Command::Axis(
                AxisId::X,
                AxisAction::MoveRelative {
                    max_velocity_in_millimeters_per_sec: 10_f64,
                    distance_in_millimeters: -40_f64,
                },
            ),
        ];

        let start_commands: [Command; 1] = [
            Command::Axis(
                AxisId::X,
                AxisAction::Home {
                    max_velocity_in_millimeters_per_sec: 10_f64,
                    back_off_distance_in_millimeters: 0.1_f64,
                },
            ),
            /*
            Command::MainSpindleSet(SpindleSetMessage {
                status: SpindleStatus::On { rpm: 1000 },
            }),
            */
        ];

        let stop_commands: [Command; 1] = [
            Command::Axis(
                AxisId::X,
                AxisAction::MoveAbsolute {
                    max_velocity_in_millimeters_per_sec: 10_f64,
                    position_in_millimeters: 0_f64,
                },
            ),
            /*
            Command::MainSpindleSet(SpindleSetMessage {
                status: SpindleStatus::Off,
            }),
            */
        ];

        Self {
            runner,
            state: MachineState::Idle,
            run_commands: Vec::from_slice(&run_commands).unwrap(),
            start_commands: Vec::from_slice(&start_commands).unwrap(),
            stop_commands: Vec::from_slice(&stop_commands).unwrap(),
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

    pub fn poll(&mut self) -> Poll<Result<(), <Runner as Actuator<RunnerAction>>::Error>> {
        match self.state {
            MachineState::Idle => Poll::Ready(Ok(())),
            MachineState::Start => {
                self.runner.receive(&RunnerAction::Reset);

                for command in self.start_commands.iter() {
                    defmt::println!("Start: {}", command);

                    self.runner.receive(&RunnerAction::Run(*command))
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

                self.runner.receive(&RunnerAction::Run(*command));

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
                Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                Poll::Pending => Poll::Pending,
            },
            MachineState::Stop => {
                self.runner.receive(&RunnerAction::Reset);

                for command in self.stop_commands.iter() {
                    defmt::println!("Stop: {}", command);

                    self.runner.receive(&RunnerAction::Run(*command));
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
