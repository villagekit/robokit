#pragma once

#include <effects/context.hpp>
#include <effects/leds.hpp>
#include <effects/clock.hpp>

namespace BotEffects {
  void setup(BotContext *context) {
    LedsEffects::setup(context);
    ClockEffects::setup(context);
  }
}