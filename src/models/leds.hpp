#pragma once

#include <mjson.h>

#include <mpark/variant.hpp>
#include <overload.hpp>

namespace LedsModel {
  struct State {
    volatile bool green = true;
    volatile bool blue = true;
    volatile bool red = true;
  };

  enum class LED_ID { GREEN, RED, BLUE };
  struct ActionToggle {
    LED_ID led_id;
  };
  using Action = mpark::variant<ActionToggle>;

  State reducer(State state, Action action) {
    mpark::visit(overload(
      [&state](const ActionToggle action) {
        switch (action.led_id) {
          case LED_ID::GREEN:
            state.green = !state.green;
            break;
          case LED_ID::BLUE:
            state.blue = !state.blue;
            break;
          case LED_ID::RED:
            state.red = !state.red;
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
      "{ %Q: %B, %Q: %B, %Q: %B }",
      "green", state->green,
      "blue", state->blue,
      "red", state->red
    );
  }
}