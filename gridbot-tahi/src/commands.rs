use fugit::ExtU32;
use robokit::{
    actuators::{axis::AxisAction, led::LedAction},
    runner::Command,
    timer::TICK_TIMER_HZ,
};

use crate::actuators::{AxisId, LedId, SpindleId};

type BotCommand = Command<LedId, TICK_TIMER_HZ, AxisId, SpindleId>;

pub fn get_run_commands() -> [BotCommand; 8] {
    [
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
    ]
}

pub fn get_start_commands() -> [BotCommand; 1] {
    [
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
    ]
}

pub fn get_stop_commands() -> [BotCommand; 1] {
    [
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
    ]
}
