#pragma once

#include <mpark/variant.hpp>
#include <overload.hpp>

namespace ClockModel {
  struct ActionTick {};
  using Action = mpark::variant<ActionTick>;

  struct State {
    uint16_t ticks = 0;
  };

  State reducer(State state, Action action) {
    mpark::visit(overload(
      [&state](const ActionTick) {
        state.ticks++;
      }
    ), action);

    return state;
  }
}