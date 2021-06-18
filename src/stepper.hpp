// Inspired by:
// - https://github.com/Stan-Reifel/SpeedyStepper
// - http://hwml.com/LeibRamp.htm

#include <Arduino.h>

#include <functional>

class Stepper
{
  // using CompletionSubscriber = std::function<void(void)>;

  public:
    enum class Status { STOPPED, RAMP_UP, MAXING, RAMP_DOWN };
    enum class Direction { Clockwise, CounterClockwise };


    constexpr static double DEFAULT_STEPS_PER_REV = 40000.0;
    constexpr static double DEFAULT_LEADSCREW_STARTS = 4.;
    constexpr static double DEFAULT_LEADSCREW_PITCH = 2.;
    constexpr static double DEFAULT_TARGET_SPEED_IN_MM_PER_SEC = 1.;
    constexpr static double DEFAULT_ACCELERATION_IN_MM_PER_SEC_PER_SEC = 0.1;
    
    constexpr static double MICROSECS_IN_SEC = 1000000.;

    HardwareTimer timer;

    const uint32_t enable_pin;
    const uint32_t direction_pin;
    const uint32_t pulse_pin;
    const double steps_per_rev;
    const double mm_per_rev;
    const double steps_per_mm;
    const double target_speed_in_steps_per_sec;
    const double acceleration_in_steps_per_sec_per_sec;

    // leib ramp
    const uint32_t acceleration_distance_in_steps;
    const uint32_t base_step_period_in_microsecs;
    const uint32_t target_step_period_in_microsecs;
    const double acceleration_multiplier;

    volatile int64_t target_position_in_steps;
    volatile int64_t current_position_in_steps;
    volatile double current_step_period_in_microsecs;

    volatile Direction current_direction;
    volatile Status current_status;

    // CompletionSubscriber completion_subscriber;

    Stepper(
      TIM_TypeDef *tim,
      uint32_t enable_pin,
      uint32_t direction_pin,
      uint32_t pulse_pin,
      double steps_per_rev = DEFAULT_STEPS_PER_REV,
      double leadscrew_starts = DEFAULT_LEADSCREW_STARTS,
      double leadscrew_pitch = DEFAULT_LEADSCREW_PITCH,
      double target_speed_in_mm_per_sec = DEFAULT_TARGET_SPEED_IN_MM_PER_SEC,
      double acceleration_in_mm_per_sec_per_sec = DEFAULT_ACCELERATION_IN_MM_PER_SEC_PER_SEC
    ):
      timer(HardwareTimer(tim)),
      enable_pin(enable_pin),
      direction_pin(direction_pin),
      pulse_pin(pulse_pin),
      steps_per_rev(steps_per_rev),
      mm_per_rev(leadscrew_starts * leadscrew_pitch),
      steps_per_mm(steps_per_rev / mm_per_rev),
      target_speed_in_steps_per_sec(target_speed_in_mm_per_sec * steps_per_mm),
      acceleration_in_steps_per_sec_per_sec(acceleration_in_mm_per_sec_per_sec * steps_per_mm),
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

    void setup() {
      pinMode(enable_pin, OUTPUT);
      pinMode(direction_pin, OUTPUT);
      pinMode(pulse_pin, OUTPUT);

      timer.attachInterrupt(std::bind(&Stepper::step, this));
    }

    Status get_next_status() {
      switch (current_status) {
        case Status::STOPPED:
          return Status::STOPPED;
        case Status::RAMP_UP:
          if (current_position_in_steps >= acceleration_distance_in_steps) {
            return Status::MAXING;
          }
          return Status::RAMP_UP;
        case Status::MAXING: {
          auto steps_remaining = target_position_in_steps - current_position_in_steps;
          if (steps_remaining - 1 <= acceleration_distance_in_steps) {
            return Status::RAMP_DOWN;
          }
          return Status::MAXING;
        }
        case Status::RAMP_DOWN:
          if (current_position_in_steps + 1 >= target_position_in_steps) {
            return Status::STOPPED;
          }
          return Status::RAMP_DOWN;
      }
    }

    // equation [23] in http://hwml.com/LeibRamp.htm
    double get_next_step_period_in_microsecs() {
      // this is bad, infinite loop to lapse watchdog timer
      if (current_status == Status::STOPPED) {
        Serial.println("ERROR: trying to calculate next step period while stopped");
        while (true) {};
      }
      if (current_status == Status::MAXING) return target_step_period_in_microsecs;

      double p = current_step_period_in_microsecs;

      double m = current_status == Status::RAMP_UP
        ? -acceleration_multiplier
        : acceleration_multiplier;

      double q = m * p * p;
      
      return p * (1 + q + (3.0 / 2.0) * q * q);
    }

    Direction get_direction_to_target_position_in_steps(int64_t target_position_in_steps) {
      if (target_position_in_steps > current_position_in_steps) {
        return Direction::Clockwise;
      } else {
        return Direction::CounterClockwise;
      }
    }


    bool is_move_completed() {
      return current_position_in_steps == target_position_in_steps;
    }

    void move_to_position_in_mm(double target_position_in_mm) {
      target_position_in_steps = round(target_position_in_mm * steps_per_mm);
      current_step_period_in_microsecs = base_step_period_in_microsecs;
      current_status = Status::RAMP_UP;
      current_direction = get_direction_to_target_position_in_steps(target_position_in_steps);
      start_moving();
    }

    void start_moving() {
      write_enable();
      write_direction();
      write_pulse();

      // TODO try timer.setPwm()
      const uint32_t interval = base_step_period_in_microsecs;
      timer.setCount(0, MICROSEC_FORMAT);
      timer.setOverflow(interval, MICROSEC_FORMAT);
      timer.resume();
    }

    void step() {
      auto next_step_period_in_microsecs = get_next_step_period_in_microsecs();
      current_step_period_in_microsecs = next_step_period_in_microsecs;

      auto next_status = get_next_status();
      current_status = next_status;

      if (next_status == Status::STOPPED) {
        timer.pause();
        write_enable(false);
      } else {
        const uint32_t interval = next_step_period_in_microsecs;
        timer.setCount(0, MICROSEC_FORMAT);
        timer.setOverflow(interval, MICROSEC_FORMAT);
        timer.refresh();
      }
    }

    void write_enable(bool should_delay = true) {
      auto enabled_signal = current_status == Status::STOPPED ? LOW : HIGH;
      digitalWrite(enable_pin, enabled_signal);

      if (should_delay) {
        // ENABLE must be ahead of DIRECTION by at least 5 microseconds
        delayMicroseconds(5);
      }
    }

    void write_direction(bool should_delay = true) {
      auto direction_signal = current_direction == Direction::CounterClockwise ? HIGH : LOW;
      digitalWrite(direction_pin, direction_signal);

      if (should_delay) {
        // DIRECTION must be ahead of PULSE by at least 6 microseconds
        delayMicroseconds(6);
      }
    }
    
    void write_pulse() {
      digitalWrite(pulse_pin, LOW);
      // PULSE width must be no less than 2.5 microseconds
      delayMicroseconds(3);
      digitalWrite(pulse_pin, HIGH);
    }
};