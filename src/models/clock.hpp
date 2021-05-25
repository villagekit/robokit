#pragma once

#include <mjson.h>

#include <mpark/variant.hpp>
#include <overload.hpp>

namespace ClockModel {
  struct ActionTick {};
  using Action = mpark::variant<ActionTick>;

  struct State {
    volatile unsigned long ticks = 0UL;
  };

  State reducer(State state, Action action) {
    mpark::visit(overload(
      [&state](const ActionTick) {
        state.ticks++;
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
}