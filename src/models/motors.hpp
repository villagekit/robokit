#pragma once

#include <mjson.h>
#include <RingBufCPP.h>

#include <mpark/variant.hpp>
#include <overload.hpp>

namespace MotorsModel {
  namespace Util {
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
  }

  namespace Queue {
    RingBufCPP<int32_t, 20 * sizeof(int32_t)> scheduled_x_positions;
  }

  enum class MotorId { X };

  struct ActionSchedule {
    MotorId id;
    double next_position_in_mm;
  };
  struct ActionProgress {
    MotorId id;
    int32_t current_position_in_steps;
  };

  using Action = mpark::variant<ActionSchedule, ActionProgress>;

  struct MotorState {
    int32_t current_position_in_steps = 0L;
    int32_t next_position_in_steps = 0L;
    double current_position_in_mm = 0.;
    double next_position_in_mm = 0.;
  };

  struct State {
    MotorState x_motor {};
  };

  State reducer(State state, Action action) {
    mpark::visit(overload(
      [&state](const ActionSchedule action) {
        switch (action.id) {
          case MotorId::X:
            state.x_motor.next_position_in_mm = action.next_position_in_mm;
            state.x_motor.next_position_in_steps = Util::mm_to_steps(action.next_position_in_mm);
            Queue::scheduled_x_positions.add(state.x_motor.next_position_in_steps);
            break;
        }
      },
      [&state](const ActionProgress action) {
        switch (action.id) {
          case MotorId::X:
            state.x_motor.current_position_in_steps = action.current_position_in_steps;
            state.x_motor.current_position_in_mm = Util::steps_to_mm(action.current_position_in_steps);
            break;
        }
      }
    ), action);

    return state;
  }

  int print_motor(mjson_print_fn_t fn, void * fndata, va_list *ap) {
    MotorState *state = va_arg(*ap, MotorState*);

    return mjson_printf(
      fn, fndata,
      "{ %Q: %d, %Q: %g, %Q: %d, %Q: %g }",
      "current_position_in_steps", state->current_position_in_steps,
      "current_position_in_mm", state->current_position_in_mm,
      "next_position_in_steps", state->next_position_in_steps,
      "next_position_in_mm", state->next_position_in_mm
    );
  }

  int print(mjson_print_fn_t fn, void * fndata, va_list *ap) {
    State *state = va_arg(*ap, State*);

    int n = 0;
    n += mjson_printf(fn, fndata, "{ ");
    n += mjson_printf(fn, fndata, "%Q: %M", "xMotor", print_motor, &(state->x_motor));
    n += mjson_printf(fn, fndata, " }");
    return n;
  }
}