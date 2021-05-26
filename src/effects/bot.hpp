#pragma once

#include <effects/context.hpp>
#include <effects/leds.hpp>
#include <effects/clock.hpp>

namespace BotEffects {
  BotContext context;

  void setup(BotStore *store) {
    BotTimer = 
    BotContext context = {
      &store,
      &timer
    };
    LedsEffects::setup(context);
    ClockEffects::setup(context);
  }
}