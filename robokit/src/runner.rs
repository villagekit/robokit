use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use heapless::Deque;

use crate::actuators::{
    axis::AxisAction, led::LedAction, spindle::SpindleAction, Actuator, ActuatorSet,
};

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

pub struct Runner<
    const LED_TIMER_HZ: u32,
    const ACTIVE_COMMMANDS_COUNT: usize,
    LedSet,
    AxisSet,
    SpindleSet,
> where
    LedSet: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    AxisSet: ActuatorSet<Action = AxisAction>,
    SpindleSet: ActuatorSet<Action = SpindleAction>,
{
    active_commands: Deque<
        Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>,
        ACTIVE_COMMMANDS_COUNT,
    >,
    leds: LedSet,
    axes: AxisSet,
    spindles: SpindleSet,
}

impl<const LED_TIMER_HZ: u32, const ACTIVE_COMMMANDS_COUNT: usize, LedSet, AxisSet, SpindleSet>
    Runner<LED_TIMER_HZ, ACTIVE_COMMMANDS_COUNT, LedSet, AxisSet, SpindleSet>
where
    LedSet: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    AxisSet: ActuatorSet<Action = AxisAction>,
    SpindleSet: ActuatorSet<Action = SpindleAction>,
{
    pub fn new(leds: LedSet, axes: AxisSet, spindles: SpindleSet) -> Self {
        Self {
            active_commands: Deque::new(),
            leds,
            axes,
            spindles,
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub enum RunnerAction<Command> {
    Run(Command),
    Reset,
}

#[derive(Clone, Copy, Debug)]
pub enum RunnerError<LedId, LedSetError, AxisId, AxisSetError, SpindleId, SpindleSetError>
where
    LedId: Debug + Format,
    LedSetError: Debug,
    AxisId: Debug + Format,
    AxisSetError: Debug,
    SpindleId: Debug + Format,
    SpindleSetError: Debug,
{
    Led(LedId, LedSetError),
    Axis(AxisId, AxisSetError),
    Spindle(SpindleId, SpindleSetError),
}

impl<const LED_TIMER_HZ: u32, const ACTIVE_COMMMANDS_COUNT: usize, LedSet, AxisSet, SpindleSet>
    Actuator for Runner<LED_TIMER_HZ, ACTIVE_COMMMANDS_COUNT, LedSet, AxisSet, SpindleSet>
where
    LedSet: ActuatorSet<Action = LedAction<LED_TIMER_HZ>>,
    AxisSet: ActuatorSet<Action = AxisAction>,
    SpindleSet: ActuatorSet<Action = SpindleAction>,
{
    type Action = RunnerAction<Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>>;
    type Error = RunnerError<
        LedSet::Id,
        LedSet::Error,
        AxisSet::Id,
        AxisSet::Error,
        SpindleSet::Id,
        SpindleSet::Error,
    >;

    fn run(
        &mut self,
        action: &RunnerAction<Command<LED_TIMER_HZ, LedSet::Id, AxisSet::Id, SpindleSet::Id>>,
    ) {
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
                Command::Led(id, _) => self.leds.poll(&id).map_err(|err| RunnerError::Led(id, err)),
                Command::Axis(id, _) => self
                    .axes
                    .poll(&id)
                    .map_err(|err| RunnerError::Axis(id, err)),
                Command::Spindle(id, _) => self
                    .spindles
                    .poll(&id)
                    .map_err(|err| RunnerError::Spindle(id, err)),
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
