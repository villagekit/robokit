# Robo Kit 🤖

## About

Firmware for simple robotic automation.

The short-term goal is to build [an automated machine for grid beam production](https://wiki.villagekit.com/en/grid-bot/concepts).

The long-term goal is to provide a [real-time interrupt-driven](https://rtic.rs) [actor-based](https://en.wikipedia.org/wiki/Actor_model) foundation for robotic automation or CNC machine control.

## Status

Under active development, but probably not useful to anyone.

If you're here and like what's happening, please say hi! 👋

## Feature Wishlist

- Command system
  - G-Code
- Actuators
  - Led
  - Linear axis
    - Stepper motor
  - Spindle
  - Pneumatic actuator
- Sensors
  - Button
  - Switch
  - Limit switch
  - Rotary encoder
  - Linear encoder
- Interfaces
  - Physical controls
  - JSON-RPC
  - Web

## Development

See [`docs/dev.md`](./docs/dev.md)

## License

Copyright 2022 Village Kit Limited

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
