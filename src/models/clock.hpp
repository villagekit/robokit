#include <mpark/variant.hpp>
#include <overload.hpp>

struct ActionClockTick {};
using ActionClock = mpark::variant<ActionClockTick>;

struct StateClock {
  uint16_t ticks = 0;
};

StateClock reducer_clock(StateClock state, ActionClock action) {
  mpark::visit(overload(
    [&state](const ActionClockTick) {
      state.ticks++;
    }
  ), action);

  return state;
}