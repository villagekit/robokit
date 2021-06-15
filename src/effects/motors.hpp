#pragma once

#include <effects/context.hpp>
#include <models/motors.hpp>

#define MOTORS_X_ENABLE_PIN D0
#define MOTORS_X_DIR_PIN D1
#define MOTORS_X_STEP_PIN D2

namespace MotorsEffects {
  HardwareTimer hw_timer = HardwareTimer(TIM8);

  enum class Direction { Clockwise, CounterClockwise };

  static double steps_per_rev = 400.0;
  static double leadscrew_starts = 4.;
  static double leadscrew_pitch = 2.;
  static double mm_per_rev = leadscrew_starts * leadscrew_pitch;
  static double steps_per_mm = steps_per_rev / mm_per_rev;
  static double mm_per_step = 1. / steps_per_mm;

  uint64_t mm_to_steps(double distance_in_mm) {
    return distance_in_mm * steps_per_mm;
  }
  double steps_to_mm(uint64_t steps) {
    return steps * mm_per_step;
  } 

  struct MotorState {
    volatile bool enabled = true;
    volatile Direction direction = Direction::Clockwise;
    volatile int32_t absolute_steps = 0L;
    volatile uint32_t total_steps = 10000UL;
    volatile uint32_t ramp_steps = 0UL;
    volatile uint32_t steps_completed = 0UL;
  };

  volatile MotorState x_motor = MotorState {};

  void step_motor(volatile MotorState *motor) {

    // step
    // - send pulse
    // - prepare for next tick (acc/deceleration)
    if (motor->enabled) {
      if (motor->steps_completed < motor->total_steps) {
        digitalWrite(MOTORS_X_STEP_PIN, LOW);
        // pulse width must be no less than 2.5 microseconds
        delayMicroseconds(3);
        digitalWrite(MOTORS_X_STEP_PIN, HIGH);

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
      x_motor.steps_completed = 0;
      bool direction = x_motor.direction == Direction::Clockwise;
      x_motor.direction = direction
        ? Direction::CounterClockwise
        : Direction::Clockwise;

      digitalWrite(MOTORS_X_ENABLE_PIN, true);
      digitalWrite(MOTORS_X_DIR_PIN, direction);
    }
  }

  void progress(void *params) {
    BotContext *context = (BotContext *) params;

    auto store = context->store;
    store->dispatch(MotorsModel::ActionProgress {
      MotorsModel::MotorId::X,
      steps_to_mm(x_motor.absolute_steps)
    });
  }

  void setup(BotContext *context) {
    pinMode(MOTORS_X_ENABLE_PIN, OUTPUT);
    pinMode(MOTORS_X_DIR_PIN, OUTPUT);
    pinMode(MOTORS_X_STEP_PIN, OUTPUT);

    const uint32_t interval = 200UL; // microseconds
    // const float frequency = (1000000.0f / interval);
    // const uint32_t timer_count = (uint32_t) 1000000 / frequency;
    hw_timer.setCount(0, MICROSEC_FORMAT);
    // hw_timer.setOverflow(timer_count, MICROSEC_FORMAT);
    hw_timer.setOverflow(interval, MICROSEC_FORMAT);
    hw_timer.attachInterrupt(std::bind(&step, context));
    hw_timer.resume();

    auto isr_timer = context->isr_timer;
    isr_timer->setInterval(1000UL, &schedule, context);
    isr_timer->setInterval(50UL, &progress, context);
  }
}