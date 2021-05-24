#pragma once

#include <Arduino.h>
#include <mjson.h>

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

  int print(mjson_print_fn_t fn, void * fndata, va_list *ap) {
    State *state = va_arg(*ap, State*);

    noInterrupts();

    int n = 0;
    n += mjson_printf(fn, fndata, "{ ");
    n += mjson_printf(fn, fndata, "%Q: %M", "leds", LedsModel::print, &(state->leds));
    n += mjson_printf(fn, fndata, ", ");
    n += mjson_printf(fn, fndata, "%Q: %M", "clock", ClockModel::print, &(state->clock));
    n += mjson_printf(fn, fndata, " }");

    interrupts();

    return n;
  }
}