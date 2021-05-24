#pragma once

#include <Arduino.h>

#include <mpark/variant.hpp>
#include <overload.hpp>

#include <models/clock.hpp>
#include <models/leds.hpp>
#include <timer.hpp>

namespace BotModel {
  struct State {
    LedsModel::State leds = LedsModel::State {};
    ClockModel::State clock = ClockModel::State {};
  };

  using Action = mpark::variant<LedsModel::Action, ClockModel::Action>;

  State reducer(State state, Action action) {
    noInterrupts();

    mpark::visit(overload(
      [&state](const LedsModel::Action a) {
        state.leds = LedsModel::reducer(state.leds, a);
      },
      [&state](const ClockModel::Action a) {
        state.clock = ClockModel::reducer(state.clock, a);
      }
    ), action);

    interrupts();

    return state;
  }
}