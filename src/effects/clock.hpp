#pragma once

#include <effects/context.hpp>

namespace ClockEffects {
  void tick(void *params) {
    BotContext *context = (BotContext *) params;
    auto store = context->store;
    store->dispatch(ClockModel::ActionTick {});
  }

  void setup(BotContext *context) {
    auto timer = context->isr_timer;
    timer->setInterval(10UL, &tick, context);
  }
}