#pragma once

#include <effects/context.hpp>

namespace ClockEffects {
  void tick(void *params) {
    BotContext *context = (BotContext*) params;
    auto store = context->store;
    store->dispatch(ClockModel::ActionTick {});
  }

  void setup(BotContext *context) {
    pinMode(LED_GREEN, OUTPUT);
    pinMode(LED_BLUE, OUTPUT);
    pinMode(LED_RED, OUTPUT);

    auto timer = context->timer;
    timer->set_interval(10L, tick, context);
  }
}