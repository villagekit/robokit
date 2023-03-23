use core::task::Poll;
use defmt::Format;
use fugit::ExtU32;
use heapless::Vec;

use crate::{
    actor::{ActorPoll, ActorReceive},
    actuators::axis::AxisMoveMessage,
    actuators::spindle::{SpindleSetMessage, SpindleStatus},
    actuators::{axis::AxisHomeMessage, led::LedBlinkMessage},
    command::{ActuatorError, Command, CommandCenter, ResetMessage, SensorError},
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

pub struct Machine {
    command_center: CommandCenter,
    state: MachineState,
    run_commands: Vec<Command, 32>,
    start_commands: Vec<Command, 4>,
    stop_commands: Vec<Command, 4>,
}

impl Machine {
    pub fn new(command_center: CommandCenter) -> Self {
        let run_commands: [Command; 8] = [
            Command::GreenLedBlink(LedBlinkMessage {
                duration: 50.millis(),
            }),
            Command::BlueLedBlink(LedBlinkMessage {
                duration: 100.millis(),
            }),
            Command::RedLedBlink(LedBlinkMessage {
                duration: 200.millis(),
            }),
            Command::XAxisMove(AxisMoveMessage {
                max_velocity_in_millimeters_per_sec: 10_f64,
                distance_in_millimeters: 120_f64,
            }),
            Command::RedLedBlink(LedBlinkMessage {
                duration: 50.millis(),
            }),
            Command::BlueLedBlink(LedBlinkMessage {
                duration: 100.millis(),
            }),
            Command::GreenLedBlink(LedBlinkMessage {
                duration: 200.millis(),
            }),
            Command::XAxisMove(AxisMoveMessage {
                max_velocity_in_millimeters_per_sec: 10_f64,
                distance_in_millimeters: -120_f64,
            }),
        ];

        let start_commands: [Command; 1] = [
            Command::XAxisHome(AxisHomeMessage {
                max_velocity_in_millimeters_per_sec: 10_f64,
            }),
            /*
            Command::MainSpindleSet(SpindleSetMessage {
                status: SpindleStatus::On { rpm: 1000 },
            }),
            */
        ];

        let stop_commands: [Command; 1] = [
            Command::XAxisHome(AxisHomeMessage {
                max_velocity_in_millimeters_per_sec: 10_f64,
            }),
            /*
            Command::MainSpindleSet(SpindleSetMessage {
                status: SpindleStatus::Off,
            }),
            */
        ];

        Self {
            command_center,
            state: MachineState::Idle,
            run_commands: Vec::from_slice(&run_commands).unwrap(),
            start_commands: Vec::from_slice(&start_commands).unwrap(),
            stop_commands: Vec::from_slice(&stop_commands).unwrap(),
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct StartMessage {}

impl ActorReceive<StartMessage> for Machine {
    fn receive(&mut self, _message: &StartMessage) {
        self.state = MachineState::Start;
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct StopMessage {}

impl ActorReceive<StopMessage> for Machine {
    fn receive(&mut self, _message: &StopMessage) {
        self.state = MachineState::Stop;
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct ToggleMessage {}

impl ActorReceive<ToggleMessage> for Machine {
    fn receive(&mut self, _message: &ToggleMessage) {
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
}

#[derive(Debug)]
pub enum MachineError {
    Actuator(ActuatorError),
    Sensor(SensorError),
}

impl ActorPoll for Machine {
    type Error = MachineError;

    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        self.command_center
            .sense()
            .map_err(|err| MachineError::Sensor(err))?;

        match self.state {
            MachineState::Idle => Poll::Ready(Ok(())),
            MachineState::Start => {
                self.command_center.receive(&ResetMessage {});

                for command in self.start_commands.iter() {
                    defmt::println!("Start: {}", command);

                    self.command_center.receive(command)
                }

                self.state = MachineState::StartLoop;

                Poll::Pending
            }
            MachineState::StartLoop => {
                if let Poll::Ready(Ok(())) = self.command_center.poll() {
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

                self.command_center.receive(command);

                self.state = MachineState::RunLoop { command_index };

                Poll::Pending
            }
            MachineState::RunLoop { command_index } => match self.command_center.poll() {
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
                Poll::Ready(Err(err)) => Poll::Ready(Err(MachineError::Actuator(err))),
                Poll::Pending => Poll::Pending,
            },
            MachineState::Stop => {
                self.command_center.receive(&ResetMessage {});

                for command in self.stop_commands.iter() {
                    defmt::println!("Stop: {}", command);

                    self.command_center.receive(command)
                }

                self.state = MachineState::StopLoop;

                Poll::Pending
            }
            MachineState::StopLoop => {
                if let Poll::Ready(Ok(())) = self.command_center.poll() {
                    self.state = MachineState::Idle
                }

                Poll::Pending
            }
        }
    }
}
