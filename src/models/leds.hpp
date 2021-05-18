#pragma once

#include <mpark/variant.hpp>
#include <overload.hpp>

namespace LedsModel {
  struct State {
    bool green = true;
    bool blue = true;
    bool red = true;
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
}