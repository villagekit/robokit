#pragma once

// http://hwml.com/LeibRamp.htm

#include <Arduino.h>

class StepperRamp
{
  public:
    constexpr static double DEFAULT_TARGET_SPEED_IN_STEPS_PER_SEC = 50.;
    constexpr static double DEFAULT_ACCELERATION_IN_STEPS_PER_SEC_PER_SEC = 25.;
    
    constexpr static double MICROSECS_IN_SEC = 1000000.;

    const double target_speed_in_steps_per_sec;
    const double acceleration_in_steps_per_sec_per_sec;

    const uint32_t acceleration_distance_in_steps;
    const uint32_t base_step_period_in_microsecs;
    const uint32_t target_step_period_in_microsecs;
    const double acceleration_multiplier;

    StepperRamp(
      double target_speed_in_steps_per_sec = DEFAULT_TARGET_SPEED_IN_STEPS_PER_SEC,
      double acceleration_in_steps_per_sec_per_sec = DEFAULT_ACCELERATION_IN_STEPS_PER_SEC_PER_SEC
    ):
      target_speed_in_steps_per_sec(target_speed_in_steps_per_sec),
      acceleration_in_steps_per_sec_per_sec(acceleration_in_steps_per_sec_per_sec),
      acceleration_distance_in_steps(
        round(
          pow(target_speed_in_steps_per_sec, 2)
          / (2 * acceleration_in_steps_per_sec_per_sec)
        )
      ),
      base_step_period_in_microsecs(
        round(MICROSECS_IN_SEC / sqrt(2. * acceleration_in_steps_per_sec_per_sec))
      ),
      target_step_period_in_microsecs(
        round(MICROSECS_IN_SEC / target_speed_in_steps_per_sec)
      ),
      acceleration_multiplier(
        acceleration_in_steps_per_sec_per_sec / pow(MICROSECS_IN_SEC, 2.)
      ) {}

    StepperRampMovement movement(uint32_t steps) {
      return StepperRampMovement(this, steps);
    }
};

class StepperRampMovement
{
  public:
    enum class Status { START, RAMP_UP, MAX, RAMP_DOWN, END };

    StepperRamp *stepper_ramp;

    volatile uint32_t steps_total;
    volatile uint32_t steps_completed;
    volatile uint32_t acceleration_steps;

    volatile double current_step_period_in_microsecs;

    StepperRampMovement(
      StepperRamp *stepper_ramp,
      uint32_t steps_total
    ):
      stepper_ramp(stepper_ramp),
      steps_total(steps_total),
      steps_completed(0),
      acceleration_steps(0),
      current_step_period_in_microsecs(
        stepper_ramp->base_step_period_in_microsecs
      ) {}

    bool is_done() {
      return status() == Status::END;
    }

    // equation [23] in http://hwml.com/LeibRamp.htm
    uint32_t step_period_in_microsecs() {
      auto current_status = status();
      if (current_status == Status::START) return stepper_ramp->base_step_period_in_microsecs;
      if (current_status == Status::END) return stepper_ramp->base_step_period_in_microsecs;
      if (current_status == Status::MAX) return stepper_ramp->target_step_period_in_microsecs;

      double p = current_step_period_in_microsecs;

      double m = current_status == Status::RAMP_UP
        ? -stepper_ramp->acceleration_multiplier
        : stepper_ramp->acceleration_multiplier;

      double q = m * p * p;
      
      double next_step_period_in_microsecs = p * (1 + q + (3.0 / 2.0) * q * q);
      
      return constrain(
        next_step_period_in_microsecs,
        stepper_ramp->target_step_period_in_microsecs,
        stepper_ramp->base_step_period_in_microsecs  
      );
    }

    void increment() {
      steps_completed++;
    }

    Status status() {
      if (steps_completed == 0) {
        return Status::START;
      }

      if (steps_completed >= steps_total) {
        return Status::END;
      }

      if (steps_completed <= acceleration_steps) {
        return Status::RAMP_UP;
      }

      auto steps_remaining = steps_total - steps_completed;
      if (steps_remaining <= acceleration_steps) {
        return Status::RAMP_DOWN;
      }

      if (steps_completed > acceleration_steps) {
        return Status::MAX;
      }

      // should never get here
      return Status::END;
    }
};