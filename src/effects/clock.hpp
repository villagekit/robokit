#pragma once

#include <functional>

#include <STM32TimerInterrupt.h>

#include <effects/context.hpp>

namespace ClockEffects {
  STM32Timer timer = STM32Timer(TIM8);

  void tick(BotContext *context) {
    auto store = context->store;
    store->dispatch(ClockModel::ActionTick {});
  }

  void setup(BotContext *context) {
    // timer.attachInterruptInterval(1300UL, std::bind(&tick, context));
  }
}