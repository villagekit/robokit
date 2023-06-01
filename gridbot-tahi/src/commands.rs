use fugit::ExtU32;
use robokit::{AxisAction, Command, LedAction};

use crate::actuators::{AxisId, LedId, SpindleId};

type BotCommand<const TIMER_HZ: u32> = Command<TIMER_HZ, LedId, AxisId, SpindleId>;

/*
pub fn get_run_commands<const TIMER_HZ: u32>() -> [BotCommand<TIMER_HZ>; 4] {
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
            LedId::Green,
            LedAction::Blink {
                duration: 500.millis(),
            },
        ),
        Command::Axis(
            AxisId::Length,
            AxisAction::MoveAbsolute {
                max_velocity_in_millimeters_per_sec: 10_f64,
                position_in_millimeters: 0_f64,
            },
        ),
    ]
}
*/

/*
pub fn get_run_commands<const TIMER_HZ: u32>() -> [BotCommand<TIMER_HZ>; 4] {
    [
        Command::Led(
            LedId::Blue,
            LedAction::Blink {
                duration: 500.millis(),
            },
        ),
        Command::Axis(
            AxisId::Width,
            AxisAction::MoveAbsolute {
                max_velocity_in_millimeters_per_sec: 1_f64,
                position_in_millimeters: 2_f64,
            },
        ),
        Command::Led(
            LedId::Blue,
            LedAction::Blink {
                duration: 500.millis(),
            },
        ),
        Command::Axis(
            AxisId::Width,
            AxisAction::MoveAbsolute {
                max_velocity_in_millimeters_per_sec: 1_f64,
                position_in_millimeters: 0_f64,
            },
        ),
    ]
}
*/

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
                max_velocity_in_millimeters_per_sec: 20_f64,
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
            AxisId::Width,
            AxisAction::MoveAbsolute {
                max_velocity_in_millimeters_per_sec: 10_f64,
                position_in_millimeters: 20_f64,
            },
        ),
        Command::Led(
            LedId::Blue,
            LedAction::Blink {
                duration: 500.millis(),
            },
        ),
        Command::Axis(
            AxisId::Width,
            AxisAction::MoveAbsolute {
                max_velocity_in_millimeters_per_sec: 10_f64,
                position_in_millimeters: 0_f64,
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
                max_velocity_in_millimeters_per_sec: 20_f64,
                position_in_millimeters: 0_f64,
            },
        ),
        Command::Led(
            LedId::Red,
            LedAction::Blink {
                duration: 500.millis(),
            },
        ),
    ]
}

pub fn get_start_commands<const TIMER_HZ: u32>() -> [BotCommand<TIMER_HZ>; 2] {
    [
        Command::Axis(
            AxisId::Width,
            AxisAction::Home {
                max_velocity_in_millimeters_per_sec: 5_f64,
                back_off_distance_in_millimeters: 1_f64,
            },
        ),
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
