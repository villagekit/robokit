<h1 align="center">Robo Kit 🤖</h1>

<div align="center">
  <strong>
    Build custom firmware for simple robotic automation
  </strong>
</div>

## Modules

- [`robokit`](./robokit) : [![Crates.io version](https://img.shields.io/crates/v/robokit.svg?style=flat-square) ](https://crates.io/crates/robokit)  [![Download](https://img.shields.io/crates/d/robokit.svg?style=flat-square)](https://crates.io/crates/robokit)  [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/robokit)

## Firmwares

- [`blinky`](./blinky)
- [`gridbot-tahi`](./gridbot-tahi)

## About

The short-term goal is to build [an automated machine for grid beam production](https://github.com/villagekit/gridbot-tahi).

The long-term goal is to provide a ([real-time interrupt-driven](https://rtic.rs)) [actor-based](https://en.wikipedia.org/wiki/Actor_model) foundation for robotic automation or CNC machine control.

If you're here and like what's happening, please give this a star and [say hi](https://github.com/villagekit/robokit/issues)! 👋

## Features

- Minimal
  - Designed for `no-std` + `alloc` environments
- Extensible
  - Setup your robot with your own actuators with your own names
      - E.g. Isn't limited to only x, y, z linear axes
- Command system (like G-Code)
  - Run a sequence of commands (one at a time)
  - Run setup commands at beginning and/or teardown commands at end (in parallel)
- Actuators:
  - [x] Led
      - Actions:
        - Blink { duration }
  - [x] Linear Axis
      - Drivers: [Stepper](https://github.com/braun-embedded/stepper)
      - Actions:
        - MoveRelative { max_acceleration, distance }
        - MoveAbsolute { max_acceleration, duration }
        - Home { max_acceleration, back_off_distance }
  - [x] Spindle
      - Drivers:
        - JmcHsv57
      - Actions:
        - Set(On { rpm })
        - Set(Off)
  - [ ] Relay (Pneumatic actuator)
- Sensors
  - [x] Input switch
      - Button
      - Limit switch
  - [ ] Rotary encoder
  - [ ] Linear encoder
- Interfaces
  - [ ] Physical controls
  - [ ] JSON-RPC
  - [ ] Web

## Example

[`./blinky/src/main.rs`](./blinky/src/main.rs)

(for [Nucleo-F767ZI](https://nz.element14.com/stmicroelectronics/nucleo-f767zi/dev-board-nucleo-32-mcu/dp/2546569))

```rust
#![no_main]
#![no_std]

use blinky as _;

use core::task::Poll;
use cortex_m_rt::entry;
use defmt::Debug2Format;
use fugit::ExtU32;
use robokit::{actuators::led::LedAction, runner::Command};
use robokit::{
    actuators::led::LedDevice,
    robot::RobotBuilder,
    sensors::{
        switch::{SwitchDevice, SwitchStatus},
        Sensor,
    },
    timer::SuperTimer,
};
use stm32f7xx_hal::{pac, prelude::*};

use blinky::init_heap;

pub const MILLIS_HZ: u32 = 1_000;

pub fn get_run_commands() -> [Command<'static, MILLIS_HZ>; 6] {
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
    ]
}

#[entry]
fn main() -> ! {
    init_heap();

    defmt::println!("Init!");

    let p = pac::Peripherals::take().unwrap();

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();

    let tick_timer = p.TIM5.counter_us(&clocks);
    let mut super_timer = SuperTimer::new(tick_timer, u32::MAX);

    let user_button_pin = gpioc.pc13.into_floating_input();
    let user_button_timer = super_timer.sub();
    let mut user_button = SwitchDevice::new_active_high(user_button_pin, user_button_timer);

    let mut robot_builder = RobotBuilder::new();

    let green_led_pin = gpiob.pb0.into_push_pull_output();
    let green_led_timer = super_timer.sub();
    let green_led = LedDevice::new(green_led_pin, green_led_timer);
    robot_builder.add_led("green", green_led).unwrap();

    let blue_led_pin = gpiob.pb7.into_push_pull_output();
    let blue_led_timer = super_timer.sub();
    let blue_led = LedDevice::new(blue_led_pin, blue_led_timer);
    robot_builder.add_led("blue", blue_led).unwrap();

    let red_led_pin = gpiob.pb14.into_push_pull_output();
    let red_led_timer = super_timer.sub();
    let red_led = LedDevice::new(red_led_pin, red_led_timer);
    robot_builder.add_led("red", red_led).unwrap();

    robot_builder.set_run_commands(&get_run_commands()).unwrap();

    let mut robot = robot_builder.build();

    loop {
        super_timer.tick().expect("Error ticking super timer");

        if let Some(user_button_update) = user_button.sense().expect("Error reading user button") {
            if let SwitchStatus::On = user_button_update.status {
                robot.toggle();
            }
        }

        if let Poll::Ready(Err(err)) = robot.poll() {
            defmt::println!("Unexpected error: {}", Debug2Format(&err));

            robot.stop();
        }
    }
}
```

## Development

See [`docs/dev.md`](./docs/dev.md)

## License

Copyright 2023 Village Kit Limited

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
