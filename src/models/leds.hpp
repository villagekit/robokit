#include <mpark/variant.hpp>
#include <overload.hpp>

struct StateLeds {
  bool green = true;
  bool blue = true;
  bool red = true;
};

enum class LED_ID { GREEN, RED, BLUE };
struct ActionLedToggle {
  LED_ID led_id;
};
using ActionLeds = mpark::variant<ActionLedToggle>;

StateLeds reducer_leds(StateLeds state, ActionLeds action) {
  mpark::visit(overload(
    [&state](const ActionLedToggle action) {
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
