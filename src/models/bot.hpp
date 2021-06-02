#pragma once

#include <Arduino.h>
#include <mjson.h>

#include <mpark/variant.hpp>
#include <overload.hpp>

#include <models/clock.hpp>
#include <models/leds.hpp>
#include <models/motors.hpp>

namespace BotModel {
  struct State {
    LedsModel::State leds = LedsModel::State {};
    ClockModel::State clock = ClockModel::State {};
    MotorsModel::State motors = MotorsModel::State {};
  };

  using Action = mpark::variant<ClockModel::Action, LedsModel::Action, MotorsModel::Action>;

  State reducer(State state, Action action) {
    mpark::visit(overload(
      [&state](const ClockModel::Action a) {
        state.clock = ClockModel::reducer(state.clock, a);
      },
      [&state](const LedsModel::Action a) {
        state.leds = LedsModel::reducer(state.leds, a);
      },
      [&state](const MotorsModel::Action a) {
        state.motors = MotorsModel::reducer(state.motors, a);
      }
    ), action);

    return state;
  }

  int print(mjson_print_fn_t fn, void * fndata, va_list *ap) {
    State *state = va_arg(*ap, State*);

    int n = 0;
    n += mjson_printf(fn, fndata, "{ ");
    n += mjson_printf(fn, fndata, "%Q: %M", "leds", LedsModel::print, &(state->leds));
    n += mjson_printf(fn, fndata, ", ");
    n += mjson_printf(fn, fndata, "%Q: %M", "clock", ClockModel::print, &(state->clock));
    n += mjson_printf(fn, fndata, ", ");
    n += mjson_printf(fn, fndata, "%Q: %M", "motors", MotorsModel::print, &(state->motors));
    n += mjson_printf(fn, fndata, " }");

    return n;
  }
}