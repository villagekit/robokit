#pragma once

#include <effects/context.hpp>
#include <models/motors.hpp>

namespace MotorsEffects {
  HardwareTimer hw_timer = HardwareTimer(TIM8);

  void step(BotContext *context) {
    auto store = context->store;
    store->dispatch(MotorsModel::ActionStep {}, false);
  }

  void setup(BotContext *context) {
    const uint32_t interval = 200UL; // microseconds
    const float frequency = (1000000.0f / interval);
    const uint32_t timer_count = (uint32_t) 1000000 / frequency;
    hw_timer.setCount(0, MICROSEC_FORMAT);
    hw_timer.setOverflow(timer_count, MICROSEC_FORMAT);
    hw_timer.attachInterrupt(std::bind(&step, context));
    hw_timer.resume();
  }
}