use alloc::boxed::Box;
use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use heapless::Vec;

use crate::{actuators::Actuator, error::Error, runner::RunnerAction};

#[derive(Clone, Copy, Debug, Format)]
pub enum SchedulerState {
    Idle,
    Start,
    StartLoop,
    Run { command_index: usize },
    RunLoop { command_index: usize },
    Stop,
    StopLoop,
}

pub struct Scheduler<
    Command,
    Runner,
    const RUN_COMMANDS_COUNT: usize,
    const START_COMMANDS_COUNT: usize,
    const STOP_COMMANDS_COUNT: usize,
> where
    Runner: Actuator<Action = RunnerAction<Command>>,
{
    runner: Runner,
    state: SchedulerState,
    run_commands: Vec<Command, RUN_COMMANDS_COUNT>,
    start_commands: Vec<Command, START_COMMANDS_COUNT>,
    stop_commands: Vec<Command, STOP_COMMANDS_COUNT>,
}

impl<
        Command,
        Runner,
        const RUN_COMMANDS_COUNT: usize,
        const START_COMMANDS_COUNT: usize,
        const STOP_COMMANDS_COUNT: usize,
    > Scheduler<Command, Runner, RUN_COMMANDS_COUNT, START_COMMANDS_COUNT, STOP_COMMANDS_COUNT>
where
    Command: Debug + Format,
    Runner: Actuator<Action = RunnerAction<Command>>,
{
    pub fn new(
        runner: Runner,
        run_commands: Vec<Command, RUN_COMMANDS_COUNT>,
        start_commands: Vec<Command, START_COMMANDS_COUNT>,
        stop_commands: Vec<Command, STOP_COMMANDS_COUNT>,
    ) -> Self {
        Self {
            runner,
            state: SchedulerState::Idle,
            run_commands,
            start_commands,
            stop_commands,
        }
    }

    pub fn start(&mut self) {
        self.state = SchedulerState::Start;
    }

    pub fn stop(&mut self) {
        self.state = SchedulerState::Stop;
    }

    pub fn toggle(&mut self) {
        self.state = match self.state {
            SchedulerState::Idle => SchedulerState::Start,
            SchedulerState::Start => SchedulerState::Stop,
            SchedulerState::StartLoop => SchedulerState::Stop,
            SchedulerState::Run { .. } => SchedulerState::Stop,
            SchedulerState::RunLoop { .. } => SchedulerState::Stop,
            SchedulerState::Stop => SchedulerState::Start,
            SchedulerState::StopLoop => SchedulerState::Start,
        };
    }

    pub fn poll(&mut self) -> Poll<Result<(), Runner::Error>> {
        match self.state {
            SchedulerState::Idle => Poll::Ready(Ok(())),
            SchedulerState::Start => {
                self.runner.run(&RunnerAction::Reset);

                for command in self.start_commands.iter() {
                    defmt::println!("Start: {}", command);

                    self.runner.run(&RunnerAction::Run(*command))
                }

                self.state = SchedulerState::StartLoop;

                Poll::Pending
            }
            SchedulerState::StartLoop => {
                if let Poll::Ready(Ok(())) = self.runner.poll() {
                    self.state = SchedulerState::Run { command_index: 0 };
                }

                Poll::Pending
            }
            SchedulerState::Run { command_index } => {
                let command = self
                    .run_commands
                    .get(command_index)
                    .expect("Unexpected run command index");

                defmt::println!("Run: {}", command);

                self.runner.run(&RunnerAction::Run(*command));

                self.state = SchedulerState::RunLoop { command_index };

                Poll::Pending
            }
            SchedulerState::RunLoop { command_index } => match self.runner.poll() {
                Poll::Ready(Ok(())) => {
                    let next_command_index = command_index + 1;

                    if next_command_index >= self.run_commands.len() {
                        self.state = SchedulerState::Stop;
                    } else {
                        self.state = SchedulerState::Run {
                            command_index: next_command_index,
                        };
                    }

                    Poll::Pending
                }
                Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                Poll::Pending => Poll::Pending,
            },
            SchedulerState::Stop => {
                self.runner.run(&RunnerAction::Reset);

                for command in self.stop_commands.iter() {
                    defmt::println!("Stop: {}", command);

                    self.runner.run(&RunnerAction::Run(*command));
                }

                self.state = SchedulerState::StopLoop;

                Poll::Pending
            }
            SchedulerState::StopLoop => {
                if let Poll::Ready(Ok(())) = self.runner.poll() {
                    self.state = SchedulerState::Idle
                }

                Poll::Pending
            }
        }
    }
}
