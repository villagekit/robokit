// Inspired by:
// - https://github.com/Stan-Reifel/SpeedyStepper
//   - see https://github.com/Stan-Reifel/SpeedyStepper/blob/master/Documentation.md
// - http://hwml.com/LeibRamp.htm

#include <functional>

#include <Arduino.h>

#include <SimplyAtomic.h>

/*
Notes:
- Once a motion starts, you cannot change the target position, speed, or
    rate of acceleration until the motion has completed.
  - The only exception is you can issue a "Stop" at any point in time.
*/
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

    volatile uint32_t movement_steps_total;
    volatile uint32_t movement_steps_completed;

    volatile Direction current_direction;
    volatile Status current_status;

    volatile bool is_paused;

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
      ),
      target_position_in_steps(0),
      current_position_in_steps(0),
      current_step_period_in_microsecs(base_step_period_in_microsecs),
      movement_steps_total(0),
      movement_steps_completed(0),
      current_direction(Direction::Clockwise),
      current_status(Status::STOPPED),
      is_paused(true)
       {
        timer.setCount(0, MICROSEC_FORMAT);
        timer.setOverflow(base_step_period_in_microsecs, MICROSEC_FORMAT);
        timer.attachInterrupt(std::bind(&Stepper::step, this));
        timer.refresh();
      }

    void setup() {
      pinMode(enable_pin, OUTPUT);
      pinMode(direction_pin, OUTPUT);
      pinMode(pulse_pin, OUTPUT);

      write_enable(false);
    }

    bool is_move_completed() {
      return movement_steps_completed == movement_steps_total;
    }

    void move_to_position_in_mm(double target_position_in_mm) {
      auto should_move = set_movement(round(target_position_in_mm * steps_per_mm));
      if (should_move) start_movement();
    }

    void move_to_position_in_steps(int64_t target_position_in_steps) {
      auto should_move = set_movement(target_position_in_steps);
      if (should_move) start_movement();
    }

    bool set_movement(int64_t target_position_in_steps) {
      if (current_position_in_steps == target_position_in_steps) return false; // early skip

      this->target_position_in_steps = target_position_in_steps;

      movement_steps_completed = 0;
      movement_steps_total = abs(target_position_in_steps - current_position_in_steps);

      current_step_period_in_microsecs = base_step_period_in_microsecs;
      current_status = Status::RAMP_UP;
      current_direction = target_position_in_steps > current_position_in_steps
        ? Direction::Clockwise
        : Direction::CounterClockwise;

      return true;
    }

    void start_movement() {
      write_enable();
      write_direction();
      write_pulse();
      increment_step();
      schedule_step(true);
    }

    void pause_movement() {
      ATOMIC() {
        timer.pause();
        is_paused = true;
      }
    }

    void resume_movement() {
      ATOMIC() {
        timer.resume();
        is_paused = false;
      }
    }

    // TODO: decelerate to a stop?
    void stop_movement() {
      current_status = Status::STOPPED;
      if (!is_paused) {
        pause_movement();
      }
      timer.setCount(0, MICROSEC_FORMAT);
      write_enable(false);
    }

    void schedule_step(bool is_initial_step) {
      const uint32_t interval = current_step_period_in_microsecs;

      timer.setCount(0, MICROSEC_FORMAT); // NOTE(mw): not sure if this is worthwhile
      timer.setOverflow(interval, MICROSEC_FORMAT);
      timer.refresh();

      resume_movement();
    }

    void step() {
      if (current_status == Status::STOPPED) {
        stop_movement();
        return;
      }

      pause_movement();

      auto previous_status = current_status;
      
      ATOMIC() {
        write_pulse();
        increment_step();
        calculate_next_step();
      }

      switch (current_status) {
        case Status::STOPPED:
          stop_movement();
          break;
        case Status::MAXING:
        case Status::RAMP_UP:
        case Status::RAMP_DOWN:
          schedule_step(false);
          break;
      }
    }

    void increment_step() {
      movement_steps_completed++;
      
      switch (current_direction) {
        case Direction::Clockwise:
          current_position_in_steps++;
          break;
        case Direction::CounterClockwise:
          current_position_in_steps--;
          break;
      }
    }

    void calculate_next_step() {
      current_status = calculate_status();
      current_step_period_in_microsecs = calculate_next_step_period_in_microsecs();
    }

    Status calculate_status() {
      switch (current_status) {
        case Status::STOPPED:
          return Status::STOPPED;
        case Status::RAMP_UP:
          if (movement_steps_completed > acceleration_distance_in_steps) {
            return Status::MAXING;
          }
          return Status::RAMP_UP;
        case Status::MAXING: {
          auto steps_remaining = movement_steps_total - movement_steps_completed;
          if (steps_remaining <= acceleration_distance_in_steps) {
            return Status::RAMP_DOWN;
          }
          return Status::MAXING;
        }
        case Status::RAMP_DOWN:
          if (movement_steps_completed >= movement_steps_total) {
            return Status::STOPPED;
          }
          return Status::RAMP_DOWN;
      }
    }

    // equation [23] in http://hwml.com/LeibRamp.htm
    double calculate_next_step_period_in_microsecs() {
      if (current_status == Status::STOPPED) return base_step_period_in_microsecs;
      if (current_status == Status::MAXING) return target_step_period_in_microsecs;

      double p = current_step_period_in_microsecs;

      double m = current_status == Status::RAMP_UP
        ? -acceleration_multiplier
        : acceleration_multiplier;

      double q = m * p * p;
      
      double next_step_period_in_microsecs = p * (1 + q + (3.0 / 2.0) * q * q);
      
      return constrain(
        next_step_period_in_microsecs,
        target_step_period_in_microsecs,
        base_step_period_in_microsecs  
      );
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
      auto direction_signal = current_direction == Direction::Clockwise ? HIGH : LOW;
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