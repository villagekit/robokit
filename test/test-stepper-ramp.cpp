#include <vector>

#include <Arduino.h>
#include <unity.h>

#include <stepper-ramp.hpp>

double target_speed_in_steps_per_sec = 50.;
double acceleration_in_steps_per_sec_per_sec = 50.;

StepperRamp stepper_ramp = StepperRamp(
  target_speed_in_steps_per_sec,
  acceleration_in_steps_per_sec_per_sec
);

uint32_t test_steps = 100;

StepperRampMovement stepper_ramp_movement = StepperRampMovement(
  &stepper_ramp,
  test_steps
);

std::vector<uint32_t> acceleration_steps {
  100000,
  87500,
  73237,
  61497,
  53167,
  47245,
  42855,
  39462,
  36748,
  34518,
  32645,
  31045,
  29657,
  28439,
  27358,
  26392,
  25521,
  24730,
  24009,
  23346,
  22736,
  22171,
  21646,
  21157,
  20699,
  20270,
};

std::vector<uint32_t> deceleration_steps {
  20412,
  20850,
  21318,
  21819,
  22357,
  22937,
  23564,
  24245,
  24989,
  25806,
  26709,
  27712,
  28838,
  30112,
  31570,
  33260,
  35253,
  37648,
  40599,
  44359,
  49368,
  56484,
  67651,
  88445,
  100000
};

void test_ramp_calculations(void) {
  TEST_ASSERT_EQUAL_DOUBLE(50., stepper_ramp.target_speed_in_steps_per_sec);
  TEST_ASSERT_EQUAL_DOUBLE(50., stepper_ramp.acceleration_in_steps_per_sec_per_sec);
  TEST_ASSERT_EQUAL_UINT32(25, stepper_ramp.acceleration_distance_in_steps);
  TEST_ASSERT_EQUAL_UINT32(100000, stepper_ramp.base_step_period_in_microsecs);
  TEST_ASSERT_EQUAL_UINT32(20000, stepper_ramp.target_step_period_in_microsecs);
  TEST_ASSERT_EQUAL_DOUBLE(5e-11, stepper_ramp.acceleration_multiplier);
}

void test_movement(void) {
  auto current_status = stepper_ramp_movement.status();
  auto current_step_period_in_microsecs = stepper_ramp_movement.step_period_in_microsecs();
  auto step_index = stepper_ramp_movement.steps_completed;
  auto steps_total = stepper_ramp_movement.steps_total;

  auto target_step_period_in_microsecs = stepper_ramp.target_step_period_in_microsecs;
  auto acceleration_distance = stepper_ramp.acceleration_distance_in_steps;

  // in case it fails later, make sure to increment now 
  stepper_ramp_movement.increment();

  if (step_index <= acceleration_distance) {
    TEST_ASSERT_EQUAL(StepperRampMovement::Status::RAMP_UP, current_status);
    TEST_ASSERT_EQUAL_UINT32(acceleration_steps[step_index], current_step_period_in_microsecs);
  } else if (steps_total - step_index <= acceleration_distance) {
    TEST_ASSERT_EQUAL(StepperRampMovement::Status::RAMP_DOWN, current_status);
    auto deceleration_index = deceleration_steps.size() - (steps_total - step_index);
    TEST_ASSERT_EQUAL_UINT32(deceleration_steps[deceleration_index], current_step_period_in_microsecs);
  } else {
    TEST_ASSERT_EQUAL(StepperRampMovement::Status::MAX, current_status);
    TEST_ASSERT_EQUAL_UINT32(target_step_period_in_microsecs, current_step_period_in_microsecs);
  }
}

void test_end(void) {
  TEST_ASSERT_EQUAL_UINT32(stepper_ramp_movement.steps_completed, stepper_ramp_movement.steps_total);
  TEST_ASSERT_EQUAL(StepperRampMovement::Status::END, stepper_ramp_movement.status());
}

void setup() {
  // NOTE!!! Wait for >2 secs
  // if board doesn't support software reset via Serial.DTR/RTS
  delay(2000);

  UNITY_BEGIN();    // IMPORTANT LINE!

  RUN_TEST(test_ramp_calculations);
}

void loop() {
  if (stepper_ramp_movement.steps_completed < stepper_ramp_movement.steps_total) {
    RUN_TEST(test_movement);
    delay(10);
  } else {
    RUN_TEST(test_end);
    UNITY_END(); // stop unit testing
  }
}