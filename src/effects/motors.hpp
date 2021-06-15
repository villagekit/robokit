#pragma once

#include <SimplyAtomic.h>

#include <effects/context.hpp>
#include <models/motors.hpp>

#define MOTORS_X_ENABLE_PIN D0
#define MOTORS_X_DIR_PIN D1
#define MOTORS_X_PULSE_PIN D2

namespace MotorsEffects {
  HardwareTimer hw_timer = HardwareTimer(TIM8);

  enum class Direction { Clockwise, CounterClockwise };

  struct MotorState {
    volatile bool enabled = true;
    volatile bool has_changed_enabled = false;
    volatile Direction direction = Direction::Clockwise;
    volatile bool has_changed_direction = false;
    volatile int32_t absolute_steps = 0L;
    volatile uint32_t total_steps = 10000UL;
    volatile uint32_t ramp_steps = 0UL;
    volatile uint32_t steps_completed = 0UL;
  };

  volatile MotorState x_motor = MotorState {};

  void step_motor(volatile MotorState *motor) {
    if (motor->has_changed_enabled) {
      digitalWrite(MOTORS_X_ENABLE_PIN, motor->enabled);
      motor->has_changed_enabled = false;
      // ENABLE must be ahead of DIRECTION by at least 5 microseconds
      delayMicroseconds(5);
    }

    if (motor->has_changed_direction) {
      bool direction = x_motor.direction == Direction::Clockwise;
      digitalWrite(MOTORS_X_DIR_PIN, direction);
      motor->has_changed_direction = false;
      // DIRECTION must be ahead of PULSE by at least 6 microseconds
      delayMicroseconds(6);
    }

    // step
    // - send pulse
    // - prepare for next tick (acc/deceleration)
    if (motor->enabled) {
      if (motor->steps_completed < motor->total_steps) {
        digitalWrite(MOTORS_X_PULSE_PIN, LOW);
        // PULSE width must be no less than 2.5 microseconds
        delayMicroseconds(3);
        digitalWrite(MOTORS_X_PULSE_PIN, HIGH);

        motor->steps_completed++;
        switch (motor->direction) {
          case Direction::Clockwise:
            motor->absolute_steps++;
            break;
          case Direction::CounterClockwise:
            motor->absolute_steps--;
            break;
        }
      }
    }
  }

  void step(void *params) {
    BotContext *context = (BotContext *) params;
    step_motor(&x_motor);
  }

  void schedule(void *params) {
    // BotContext *context = (BotContext *) params;

    if (x_motor.steps_completed == x_motor.total_steps) {
      if (!MotorsModel::Queue::scheduled_x_positions.isEmpty()) {
        ATOMIC() {
          x_motor.enabled = true;
          x_motor.has_changed_enabled = true;

          int32_t next_position_in_steps;
          MotorsModel::Queue::scheduled_x_positions.pull(&next_position_in_steps);
          auto step_difference = next_position_in_steps - x_motor.absolute_steps;

          x_motor.direction = step_difference < 0
            ? Direction::CounterClockwise
            : Direction::Clockwise;
          x_motor.has_changed_direction = true;

          x_motor.total_steps = abs(step_difference);
          // x_motor.ramp_steps = 100UL
          x_motor.steps_completed = 0UL;
        };
      }
    }
  }

  void progress(void *params) {
    BotContext *context = (BotContext *) params;

    auto store = context->store;
    store->dispatch(MotorsModel::ActionProgress {
      MotorsModel::MotorId::X,
      x_motor.absolute_steps
    });
  }

  void setup(BotContext *context) {
    pinMode(MOTORS_X_ENABLE_PIN, OUTPUT);
    pinMode(MOTORS_X_DIR_PIN, OUTPUT);
    pinMode(MOTORS_X_PULSE_PIN, OUTPUT);

    const uint32_t interval = 20UL; // microseconds
    hw_timer.setCount(0, MICROSEC_FORMAT);
    hw_timer.setOverflow(interval, MICROSEC_FORMAT);
    hw_timer.attachInterrupt(std::bind(&step, context));
    hw_timer.resume();

    auto isr_timer = context->isr_timer;
    isr_timer->setInterval(1UL, &schedule, context);
    isr_timer->setInterval(1UL, &progress, context);

    auto store = context->store;
    store->subscribe([store](const BotModel::State state) {
      if (state.motors.x_motor.current_position_in_steps == state.motors.x_motor.next_position_in_steps) {
        // schedule next step
        double next_position_in_mm = state.motors.x_motor.current_position_in_steps != 0 ? 0. : 20.;
        store->dispatch(MotorsModel::ActionSchedule {
          MotorsModel::MotorId::X,
          next_position_in_mm
        });
      }
    });
  }
}