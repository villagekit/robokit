#include <Arduino.h>

#include <mpark/variant.hpp>
#include <overload.hpp>

#include <models/clock.hpp>
#include <models/leds.hpp>

struct StateBot {
  StateLeds leds = StateLeds {};
  StateClock clock = StateClock {};
};

using ActionBot = mpark::variant<ActionLeds, ActionClock>;

StateBot reducer_bot(StateBot state, ActionBot action) {
  noInterrupts();

  mpark::visit(overload(
    [&state](const ActionLeds a) {
      state.leds = reducer_leds(state.leds, a);
    },
    [&state](const ActionClock a) {
      state.clock = reducer_clock(state.clock, a);
    }
  ), action);

  interrupts();

  return state;
}