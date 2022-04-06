#include <vector>

#include <Arduino.h>
#include <unity.h>

#include <stepper.hpp>

double steps_per_rev = 400.;
double leadscrew_starts = 4.;
double leadscrew_pitch = 2.;
double target_speed_in_mm_per_sec = 1.;
double acceleration_in_mm_per_sec_per_sec = 1.;

Stepper stepper = Stepper(
  TIM1,
  D0,
  D1,
  D2,
  steps_per_rev,
  leadscrew_starts,
  leadscrew_pitch,
  target_speed_in_mm_per_sec,
  acceleration_in_mm_per_sec_per_sec
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

void test_initial_calculations(void) {
  TEST_ASSERT_EQUAL_DOUBLE(50., stepper.steps_per_mm);
  TEST_ASSERT_EQUAL_DOUBLE(50., stepper.target_speed_in_steps_per_sec);
  TEST_ASSERT_EQUAL_DOUBLE(50., stepper.acceleration_in_steps_per_sec_per_sec);
  TEST_ASSERT_EQUAL_UINT32(25, stepper.acceleration_distance_in_steps);
  TEST_ASSERT_EQUAL_UINT32(100000, stepper.base_step_period_in_microsecs);
  TEST_ASSERT_EQUAL_UINT32(20000, stepper.target_step_period_in_microsecs);
  TEST_ASSERT_EQUAL_DOUBLE(5e-11, stepper.acceleration_multiplier);
}

void test_step(void) {
  TEST_ASSERT_EQUAL(stepper.movement_steps_completed, stepper.current_position_in_steps);

  auto current_status = stepper.current_status;
  auto current_step_period_in_microsecs = stepper.current_step_period_in_microsecs;
  auto step_index = stepper.movement_steps_completed;
  auto steps_total = stepper.movement_steps_total;
  auto acceleration_distance = stepper.acceleration_distance_in_steps;

  // in case it fails later, make sure to increment now 
  stepper.increment_step();
  stepper.calculate_next_step();

  if (step_index <= acceleration_distance) {
    TEST_ASSERT_EQUAL(Stepper::Status::RAMP_UP, current_status);
    TEST_ASSERT_EQUAL_UINT32(acceleration_steps[step_index], current_step_period_in_microsecs);
  } else if (steps_total - step_index <= acceleration_distance) {
    TEST_ASSERT_EQUAL(Stepper::Status::RAMP_DOWN, current_status);
    auto deceleration_index = deceleration_steps.size() - (steps_total - step_index);
    TEST_ASSERT_EQUAL_UINT32(deceleration_steps[deceleration_index], current_step_period_in_microsecs);
  } else {
    TEST_ASSERT_EQUAL(Stepper::Status::MAXING, current_status);
    TEST_ASSERT_EQUAL_UINT32(stepper.target_step_period_in_microsecs, current_step_period_in_microsecs);
  }
}

void test_end(void) {
  TEST_ASSERT_EQUAL_UINT32(stepper.movement_steps_completed, stepper.movement_steps_total);
  TEST_ASSERT_EQUAL_UINT64(stepper.current_position_in_steps, stepper.target_position_in_steps);
  TEST_ASSERT_EQUAL(Stepper::Status::STOPPED, stepper.current_status);
}

void setup() {
  // NOTE!!! Wait for >2 secs
  // if board doesn't support software reset via Serial.DTR/RTS
  delay(2000);

  UNITY_BEGIN();    // IMPORTANT LINE!

  RUN_TEST(test_initial_calculations);

  stepper.set_movement(100);
}

void loop() {
  if (stepper.movement_steps_completed < stepper.movement_steps_total) {
    RUN_TEST(test_step);
    delay(10);
  } else {
    RUN_TEST(test_end);
    UNITY_END(); // stop unit testing
  }
}