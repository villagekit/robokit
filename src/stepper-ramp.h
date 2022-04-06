#pragma once

// http://hwml.com/LeibRamp.htm

#include <Arduino.h>

// forward declaration
class StepperRampMovement;

class StepperRamp
{
  public:
    constexpr static double DEFAULT_TARGET_SPEED_IN_STEPS_PER_SEC = 50.;
    constexpr static double DEFAULT_ACCELERATION_IN_STEPS_PER_SEC_PER_SEC = 25.;
    
    constexpr static double MICROSECS_IN_SEC = 1000000.;

    const double target_speed_in_steps_per_sec;
    const double acceleration_in_steps_per_sec_per_sec;

    const uint32_t max_acceleration_distance_in_steps;
    const uint32_t base_step_period_in_microsecs;
    const uint32_t target_step_period_in_microsecs;
    const double acceleration_multiplier;

    StepperRamp(
      double target_speed_in_steps_per_sec = DEFAULT_TARGET_SPEED_IN_STEPS_PER_SEC,
      double acceleration_in_steps_per_sec_per_sec = DEFAULT_ACCELERATION_IN_STEPS_PER_SEC_PER_SEC
    ):
      target_speed_in_steps_per_sec(target_speed_in_steps_per_sec),
      acceleration_in_steps_per_sec_per_sec(acceleration_in_steps_per_sec_per_sec),
      max_acceleration_distance_in_steps(
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

  StepperRampMovement movement(uint32_t steps);
};

class StepperRampMovement
{
  public:
    enum class Status { START, RAMP_UP, MAX, RAMP_DOWN, END };

    StepperRamp *stepper_ramp;

    volatile uint32_t steps_total;
    volatile uint32_t steps_completed;
    volatile uint32_t acceleration_distance_in_steps;

    volatile Status current_status;
    volatile double current_step_period_in_microsecs;

    StepperRampMovement(
      StepperRamp *stepper_ramp,
      uint32_t steps_total
    ):
      stepper_ramp(stepper_ramp),
      steps_total(steps_total),
      steps_completed(0),
      acceleration_distance_in_steps(
        min(
          stepper_ramp->max_acceleration_distance_in_steps,
          steps_total / 2
        )
      ),
      current_step_period_in_microsecs(
        stepper_ramp->base_step_period_in_microsecs
      ) {}


    bool is_done();
    uint32_t next();

  protected:
    Status calculate_status();
    double calculate_next_step_period_in_microsecs();
};