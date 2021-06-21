#pragma once

#include <SimplyAtomic.h>

#include <effects/context.hpp>
#include <models/motors.hpp>
#include <stepper.hpp>

namespace MotorsEffects {
  enum class Direction { Clockwise, CounterClockwise };

  double steps_per_rev = 40000.;
  double leadscrew_starts = 4.;
  double leadscrew_pitch = 2.;
  double target_speed_in_mm_per_sec = 1.;
  double acceleration_in_mm_per_sec_per_sec = 1.;

  Stepper x_motor = Stepper(
    TIM9, D0, D1, D2,
    steps_per_rev,
    leadscrew_starts,
    leadscrew_pitch,
    target_speed_in_mm_per_sec,
    acceleration_in_mm_per_sec_per_sec
  );

  void setup(BotContext *context) {
    x_motor.setup();
  }

  void loop(BotStore *store) {
    if (x_motor.is_move_completed()) {
      // schedule next step
      int64_t next_position_in_steps = x_motor.current_position_in_steps != 0 ? 0 : 1e4;
      x_motor.move_to_position_in_steps(next_position_in_steps);
      store->dispatch(MotorsModel::ActionSchedule {
        MotorsModel::MotorId::X,
        next_position_in_steps
      });
    } else {
      store->dispatch(MotorsModel::ActionProgress {
        MotorsModel::MotorId::X,
        x_motor.current_position_in_steps
      });
    }
  }
}