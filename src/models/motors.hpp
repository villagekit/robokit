#pragma once

#include <mjson.h>

#include <mpark/variant.hpp>
#include <overload.hpp>

// queue commands?
// queue motor movements?
// keep state of motors?
//   - enabled
//   - direction
//   - steps


namespace MotorsModel {
  enum class Direction { CW, CCW };

  struct ActionTick {};
  struct ActionSetX {};

  using Action = mpark::variant<ActionTick>;

  struct State {
    volatile bool x_enabled = false;
    volatile Direction x_direction = Direction::CW;
    volatile uint32_t x_steps = 0UL;
  };

  State reducer(State state, Action action) {
    mpark::visit(overload(
      [&state](const ActionTick) {
        if (state.x_enabled) {
          state.x_steps--;
          if (state.x_steps == 0) state.x_enabled = false;
        }
      },
      [&state](const ActionTick action) {
        state.x_ticks = 
      }
    ), action);

    return state;
  }

  int print(mjson_print_fn_t fn, void * fndata, va_list *ap) {
    State *state = va_arg(*ap, State*);
    return mjson_printf(
      fn, fndata,
      "{ %Q: %lu }",
      "ticks", state->ticks
    );
  }

  unsigned long mm_to_steps(double distance_in_mm) {
    static double steps_per_rev = 40000.0;
    static double leadscrew_starts = 4.;
    static double leadscrew_pitch = 2.;
    static double mm_per_rev = leadscrew_starts * leadscrew_pitch;
    static double steps_per_mm = steps_per_rev / mm_per_rev;
    return distance_in_mm * steps_per_mm;
  }
}

// tick
// - send pulse
// - prepare for next tick (acc/deceleration)