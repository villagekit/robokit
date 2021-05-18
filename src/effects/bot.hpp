#pragma once

#include <effects/context.hpp>
#include <effects/leds.hpp>

namespace BotEffects {
  void setup(BotContext *context) {
    LedsEffects::setup(context);
  }
}