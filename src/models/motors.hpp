#pragma once

#include <inttypes.h>

#include <mjson.h>
#include <RingBufCPP.h>

#include <mpark/variant.hpp>
#include <overload.hpp>

namespace MotorsModel {
  enum class MotorId { X };

  struct ActionSchedule {
    MotorId id;
    int64_t next_position_in_steps;
  };
  struct ActionProgress {
    MotorId id;
    int64_t current_position_in_steps;
  };

  using Action = mpark::variant<ActionSchedule, ActionProgress>;

  struct MotorState {
    int64_t current_position_in_steps = 0LL;
    int64_t next_position_in_steps = 0LL;
    // double current_position_in_mm = 0.;
    // double next_position_in_mm = 0.;
  };

  struct State {
    MotorState x_motor {};
  };

  State reducer(State state, Action action) {
    mpark::visit(overload(
      [&state](const ActionSchedule action) {
        switch (action.id) {
          case MotorId::X:
            state.x_motor.next_position_in_steps = action.next_position_in_steps;
            // state.x_motor.next_position_in_mm = Util::steps_to_mm(action.next_position_in_steps);
            break;
        }
      },
      [&state](const ActionProgress action) {
        switch (action.id) {
          case MotorId::X:
            state.x_motor.current_position_in_steps = action.current_position_in_steps;
            // state.x_motor.current_position_in_mm = Util::steps_to_mm(action.current_position_in_steps);
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
      "{ %Q: %s, %Q: %s }",
      // "{ %Q: %s, %Q: %g, %Q: %s, %Q: %g }",
      "current_position_in_steps", printf("%" PRIi64, state->current_position_in_steps),
      // "current_position_in_mm", state->current_position_in_mm,
      "next_position_in_steps", printf("%" PRIi64, state->next_position_in_steps)
      // "next_position_in_mm", state->next_position_in_mm
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