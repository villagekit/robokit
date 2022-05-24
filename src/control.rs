use defmt::Format;

use crate::{
    actor::{ActorPoll, ActorReceive},
    actuators::axis::AxisMoveMessage,
    actuators::led::LedBlinkMessage,
    actuators::spindle::{SpindleSetMessage, SpindleStatus},
    command::{Command, CommandCenter},
};

static run_commands: [Command; 8] = [
    Command::GreenLed(LedBlinkMessage {
        duration: 50000.micros(),
    }),
    Command::BlueLed(LedBlinkMessage {
        duration: 50000.micros(),
    }),
    Command::RedLed(LedBlinkMessage {
        duration: 50000.micros(),
    }),
    Command::XAxis(AxisMoveMessage {
        max_velocity_in_millimeters_per_sec: 40_f64,
        distance_in_millimeters: 40_f64,
    }),
    Command::RedLed(LedBlinkMessage {
        duration: 50000.micros(),
    }),
    Command::BlueLed(LedBlinkMessage {
        duration: 50000.micros(),
    }),
    Command::GreenLed(LedBlinkMessage {
        duration: 50000.micros(),
    }),
    Command::XAxis(AxisMoveMessage {
        max_velocity_in_millimeters_per_sec: 40_f64,
        distance_in_millimeters: -40_f64,
    }),
];

static start_commands: [Command; 1] = [
    Command::MainSpindle(SpindleSetMessage {
        status: SpindleStatus::On { rpm: 1000 },
    }),
];

static stop_commands: [Command; 1] = [
    Command::MainSpindle(SpindleSetMessage {
        status: SpindleStatus::Off,
    }),
];

#[derive(Clone, Copy, Debug, Format)]
pub enum ControlState {
    Idle,
    Start,
    Run {
        command_index: usize,
    },
    RunLoop {
        command_index: usize,
    },
    Stop,
    StopLoop,
}

pub struct Control {
    command_center: CommandCenter,
    state: ControlState,
}

impl Control {
    pub fn new(command_center: CommandCenter) -> Self {
        Self {
            command_center,
            state: ControlState::Idle,
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct StartMessage {}

impl ActorReceive<StartMessage> for Control {
    fn receive(&mut self, message: &StartMessage) {
        for command in start_commands {
            self.command_center.receive(&command)
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct StopMessage {}

impl ActorReceive<StopMessage> for Control {
    fn receive(&mut self, message: &StopMessage) {
        for command in stop_commands {
            self.command_center.receive(&command)
        }
    }
}

impl ActorPoll for Control {
    fn poll(&mut self) -> Poll<Result<(), ()> {
        match self.state {
            ControlState::Idle => Poll::Ready(Ok()),
            ControlState::Start => {
                if Poll::Ready(Ok()) = self.command_center.poll() {
                    self.state = ControlState::Run {
                        command_index: 0
                    };
                    Poll::Pending
                }
            },
            ControlState::Run { control_index } => {
                let command = &run_commands[command_index];

                defmt::println!("Command: {}", command);

                self.command_center.receive(command);

                self.
            }

        }


        loop {
            match command_center.update() {
                Err(err) => {
                    defmt::panic!("Unexpected sensor error: {:?}", Debug2Format(&err));
                }
                Ok(()) => {}
            }

            match command_center.poll() {
                Poll::Ready(Err(err)) => {
                    defmt::panic!("Unexpected actuator error: {:?}", Debug2Format(&err));
                }
                Poll::Ready(Ok(())) => {
                    break;
                }
                Poll::Pending => {}
            }

            iwdg.feed();
        }

        command_index = (command_index + 1) % commands.len();
    }

        loop {
