// http://hwml.com/LeibRamp.htm

#include <stepper-ramp.h>

StepperRampMovement StepperRamp::movement(uint32_t steps) {
  return StepperRampMovement(this, steps);
}

bool StepperRampMovement::is_done() {
  current_status = calculate_status();
  return current_status == Status::END;
}

uint32_t StepperRampMovement::next() {
  current_status = calculate_status();
  current_step_period_in_microsecs = calculate_next_step_period_in_microsecs();
  
  steps_completed++;
  
  return (uint32_t) current_step_period_in_microsecs;
}

// equation [23] in http://hwml.com/LeibRamp.htm
double StepperRampMovement::calculate_next_step_period_in_microsecs() {
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
    (double) stepper_ramp->target_step_period_in_microsecs,
    (double) stepper_ramp->base_step_period_in_microsecs  
  );
}

StepperRampMovement::Status StepperRampMovement::calculate_status() {
  if (steps_completed == 0) {
    return Status::START;
  }

  if (steps_completed >= steps_total) {
    return Status::END;
  }

  if (steps_completed <= acceleration_distance_in_steps) {
    return Status::RAMP_UP;
  }

  auto steps_remaining = steps_total - steps_completed;
  if (steps_remaining <= acceleration_distance_in_steps) {
    return Status::RAMP_DOWN;
  }

  if (steps_completed > acceleration_distance_in_steps) {
    return Status::MAX;
  }

  // should never get here
  return Status::END;
}