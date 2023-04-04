use fugit::ExtU32;
use robokit::{
    actuators::{axis::AxisAction, led::LedAction},
    runner::Command,
};

type BotCommand<const TIMER_HZ: u32> = Command<'static, TIMER_HZ>;

pub fn get_run_commands<const TIMER_HZ: u32>() -> [BotCommand<TIMER_HZ>; 8] {
    [
        Command::Led(
            "green",
            LedAction::Blink {
                duration: 50.millis(),
            },
        ),
        Command::Led(
            "blue",
            LedAction::Blink {
                duration: 100.millis(),
            },
        ),
        Command::Led(
            "red",
            LedAction::Blink {
                duration: 200.millis(),
            },
        ),
        Command::Axis(
            "axis",
            AxisAction::MoveRelative {
                max_velocity_in_millimeters_per_sec: 10_f64,
                distance_in_millimeters: 40_f64,
            },
        ),
        Command::Led(
            "red",
            LedAction::Blink {
                duration: 50.millis(),
            },
        ),
        Command::Led(
            "blue",
            LedAction::Blink {
                duration: 100.millis(),
            },
        ),
        Command::Led(
            "green",
            LedAction::Blink {
                duration: 200.millis(),
            },
        ),
        Command::Axis(
            "x",
            AxisAction::MoveRelative {
                max_velocity_in_millimeters_per_sec: 10_f64,
                distance_in_millimeters: -40_f64,
            },
        ),
    ]
}

pub fn get_start_commands<const TIMER_HZ: u32>() -> [BotCommand<TIMER_HZ>; 1] {
    [
        Command::Axis(
            "x",
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

pub fn get_stop_commands<const TIMER_HZ: u32>() -> [BotCommand<TIMER_HZ>; 1] {
    [
        Command::Axis(
            "x",
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
