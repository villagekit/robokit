use fugit::ExtU32;
use robokit::{AxisAction, Command, LedAction};

use crate::actuators::{AxisId, LedId, SpindleId};

type BotCommand<const TIMER_HZ: u32> = Command<TIMER_HZ, LedId, AxisId, SpindleId>;

pub fn get_run_commands<const TIMER_HZ: u32>() -> [BotCommand<TIMER_HZ>; 9] {
    [
        Command::Led(
            LedId::Green,
            LedAction::Blink {
                duration: 500.millis(),
            },
        ),
        Command::Axis(
            AxisId::Length,
            AxisAction::MoveAbsolute {
                max_velocity_in_millimeters_per_sec: 10_f64,
                position_in_millimeters: 40_f64,
            },
        ),
        Command::Led(
            LedId::Blue,
            LedAction::Blink {
                duration: 500.millis(),
            },
        ),
        Command::Axis(
            AxisId::Length,
            AxisAction::MoveAbsolute {
                max_velocity_in_millimeters_per_sec: 10_f64,
                position_in_millimeters: 80_f64,
            },
        ),
        Command::Led(
            LedId::Red,
            LedAction::Blink {
                duration: 500.millis(),
            },
        ),
        Command::Axis(
            AxisId::Length,
            AxisAction::MoveAbsolute {
                max_velocity_in_millimeters_per_sec: 10_f64,
                position_in_millimeters: 120_f64,
            },
        ),
        Command::Led(
            LedId::Red,
            LedAction::Blink {
                duration: 166.millis(),
            },
        ),
        Command::Led(
            LedId::Blue,
            LedAction::Blink {
                duration: 166.millis(),
            },
        ),
        Command::Led(
            LedId::Green,
            LedAction::Blink {
                duration: 166.millis(),
            },
        ),
    ]
}

pub fn get_start_commands<const TIMER_HZ: u32>() -> [BotCommand<TIMER_HZ>; 1] {
    [
        Command::Axis(
            AxisId::Length,
            AxisAction::Home {
                max_velocity_in_millimeters_per_sec: 10_f64,
                back_off_distance_in_millimeters: 2_f64,
            },
        ),
        /*
        Command::MainSpindleSet(SpindleSetMessage {
            status: SpindleStatus::On { rpm: 1000 },
        }),
        */
    ]
}

pub fn get_stop_commands<const TIMER_HZ: u32>() -> [BotCommand<TIMER_HZ>; 1] {
    [
        Command::Axis(
            AxisId::Length,
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
