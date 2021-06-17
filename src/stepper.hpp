// Inspired by:
// - https://github.com/Stan-Reifel/SpeedyStepper
// - http://hwml.com/LeibRamp.htm

#include <Arduino.h>

#include <functional>

class Stepper
{
  using CompletionSubscriber = std::function<void(void)>;
  enum class Status { STOPPED, STARTED };
  enum class RampStatus { ACCELERATING, MAXING, DECELERATING };

  private:
    uint32_t enable_pin;
    uint32_t direction_pin;
    uint32_t pulse_pin;
    double steps_per_rev;
    double steps_per_mm;
    double target_speed_in_steps_per_sec;
    double acceleration_in_steps_per_sec_per_sec;

    // leib ramp
    double acceleration_distance_in_steps;
    double base_step_period_in_microsecs;
    double target_step_period_in_microsecs; // for assertions
    double acceleration_multiplier;

    long target_position_in_steps;
    long current_position_in_steps;
    double current_step_period_in_microsecs;

    CompletionSubscriber completion_subscriber;

  public:
    const static double DEFAULT_STEPS_PER_REV = 40000.0;
    const static double DEFAULT_LEADSCREW_STARTS = 4.;
    const static double DEFAULT_LEADSCREW_PITCH = 2.;
    const static double DEFAULT_TARGET_SPEED_IN_MM_PER_SEC = 1.;
    const static double DEFAULT_ACCELERATION_IN_MM_PER_SEC_PER_SEC = 0.1;
    
    const static double MICROSECS_IN_SEC = 1000000.;

    Stepper(
      uint32_t enable_pin,
      uint32_t direction_pin,
      uint32_t pulse_pin,
      double steps_per_rev = DEFAULT_STEPS_PER_REV,
      double leadscrew_starts = DEFAULT_LEADSCREW_STARTS,
      double leadscrew_pitch = DEFAULT_LEADSCREW_PITCH,
      double target_speed_in_mm_per_sec = DEFAULT_TARGET_SPEED_IN_MM_PER_SEC,
      double acceleration_in_mm_per_sec_per_sec = DEFAULT_ACCELERATION_IN_MM_PER_SEC_PER_SEC
    ):
      enable_pin(enable_pin),
      direction_pin(direction_pin),
      pulse_pin(pulse_pin),
      steps_per_rev(steps_per_rev),
    {
      double mm_per_rev = leadscrew_starts * leadscrew_pitch;
      steps_per_mm = steps_per_rev / mm_per_rev;

      target_speed_in_steps_per_sec = target_speed_in_mm_per_sec * steps_per_mm;
      acceleration_in_steps_per_sec_per_sec = acceleration_in_mm_per_sec_per_sec * steps_per_mm;

      double base_speed_in_steps_per_sec = 0; // since we only accelerate from stop.
      acceleration_distance_in_steps =
        (pow(target_speed_in_steps_per_sec, 2) - pow(base_speed_in_steps_per_sec, 2))
          / (2 * acceleration_in_steps_per_sec_per_sec);
      base_step_period_in_microsecs = MICROSECS_IN_SEC / sqrt(2. * acceleration_in_steps_per_sec_per_sec);
      target_step_period_in_microsecs = MICROSECS_IN_SEC / target_speed_in_steps_per_sec;
      acceleration_multiplier = MICROSECS_IN_SEC / pow(acceleration_in_steps_per_sec_per_sec, 2.);
    }

    RampStatus get_ramp_status() {
      if (current_position_in_steps < acceleration_distance_in_steps) {
        return RampStatus::ACCELERATING;
      } else if (target_position_in_steps - current_position_in_steps < acceleration_distance_in_steps) {
        return RampStatus::DECELERATING;
      } else {
        return RampStatus::MAXING;
      }
    }

    double get_next_step_period() {

    }

    /*

    void connectToPins(byte stepPinNumber, byte directionPinNumber);
    
    void setStepsPerMillimeter(float motorStepPerMillimeter);
    float getCurrentPositionInMillimeters();
    void setCurrentPositionInMillimeters(float currentPositionInMillimeter);
    void setSpeedInMillimetersPerSecond(float speedInMillimetersPerSecond);
    void setAccelerationInMillimetersPerSecondPerSecond(float accelerationInMillimetersPerSecondPerSecond);
    bool moveToHomeInMillimeters(long directionTowardHome, float speedInMillimetersPerSecond, long maxDistanceToMoveInMillimeters, int homeLimitSwitchPin);
    void moveRelativeInMillimeters(float distanceToMoveInMillimeters);
    void setupRelativeMoveInMillimeters(float distanceToMoveInMillimeters);
    void moveToPositionInMillimeters(float absolutePositionToMoveToInMillimeters);
    void setupMoveInMillimeters(float absolutePositionToMoveToInMillimeters);
    float getCurrentVelocityInMillimetersPerSecond();
    

    void setStepsPerRevolution(float motorStepPerRevolution);
    float getCurrentPositionInRevolutions();
    void setSpeedInRevolutionsPerSecond(float speedInRevolutionsPerSecond);
    void setCurrentPositionInRevolutions(float currentPositionInRevolutions);
    void setAccelerationInRevolutionsPerSecondPerSecond(float accelerationInRevolutionsPerSecondPerSecond);
    bool moveToHomeInRevolutions(long directionTowardHome, float speedInRevolutionsPerSecond, long maxDistanceToMoveInRevolutions, int homeLimitSwitchPin);
    void moveRelativeInRevolutions(float distanceToMoveInRevolutions);
    void setupRelativeMoveInRevolutions(float distanceToMoveInRevolutions);
    void moveToPositionInRevolutions(float absolutePositionToMoveToInRevolutions);
    void setupMoveInRevolutions(float absolutePositionToMoveToInRevolutions);
    float getCurrentVelocityInRevolutionsPerSecond();

    void enableStepper(void);
    void disableStepper(void);
    void setCurrentPositionInSteps(long currentPositionInSteps);
    long getCurrentPositionInSteps();
    void setupStop();
    void setSpeedInStepsPerSecond(float speedInStepsPerSecond);
    void setAccelerationInStepsPerSecondPerSecond(float accelerationInStepsPerSecondPerSecond);
    bool moveToHomeInSteps(long directionTowardHome, float speedInStepsPerSecond, long maxDistanceToMoveInSteps, int homeSwitchPin);
    void moveRelativeInSteps(long distanceToMoveInSteps);
    void setupRelativeMoveInSteps(long distanceToMoveInSteps);
    void moveToPositionInSteps(long absolutePositionToMoveToInSteps);
    void setupMoveInSteps(long absolutePositionToMoveToInSteps);
    bool motionComplete();
    float getCurrentVelocityInStepsPerSecond(); 
    bool processMovement(void);

  private:
    //
    // private member variables
    //
    uint32_t enablePin;
    uint32_t stepPin;
    uint32_t directionPin;
    float desiredSpeed_InStepsPerSecond;
    float acceleration_InStepsPerSecondPerSecond;
    long targetPosition_InSteps;
    float stepsPerMillimeter;
    float stepsPerRevolution;
    bool startNewMove;
    float desiredStepPeriod_InUS;
    long decelerationDistance_InSteps;
    int direction_Scaler;
    float ramp_InitialStepPeriod_InUS;
    float ramp_NextStepPeriod_InUS;
    unsigned long ramp_LastStepTime_InUS;
    float acceleration_InStepsPerUSPerUS;
    float currentStepPeriod_InUS;
    long currentPosition_InSteps;
};
