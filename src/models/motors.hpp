#pragma once

#include <mjson.h>

#include <mpark/variant.hpp>
#include <overload.hpp>

namespace MotorsModel {
  enum class MotorId { X };

  struct ActionSchedule {
    MotorId id;
    double position_in_mm;
  };
  struct ActionProgress {
    MotorId id;
    double position_in_mm;
  };

  using Action = mpark::variant<ActionSchedule, ActionProgress>;

  struct State {
    volatile double x_current_position_in_mm;
    volatile double x_next_position_in_mm;
  };

  State reducer(State state, Action action) {
    mpark::visit(overload(
      [&state](const ActionSchedule action) {
        switch (action.id) {
          case MotorId::X:
            state.x_next_position_in_mm = action.position_in_mm;
            break;
        }
      },
      [&state](const ActionProgress action) {
        switch (action.id) {
          case MotorId::X:
            state.x_current_position_in_mm = action.position_in_mm;
            break;
        }
      }
    ), action);

    return state;
  }

  int print(mjson_print_fn_t fn, void * fndata, va_list *ap) {
    State *state = va_arg(*ap, State*);
    return mjson_printf(
      fn, fndata,
      "{ %Q: %g, %Q: %g }",
      "xCurrentPositionInMm", state->x_current_position_in_mm,
      "xNextPositionInMm", state->x_next_position_in_mm
    );
  }
}