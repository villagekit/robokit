#pragma once

#include <Arduino.h>
#include <STM32_ISR_Timer.h>

#include <effects/context.hpp>

namespace LedsEffects {
  void output(void *params) {
    BotContext *context = (BotContext*) params;
    auto store = context->store;
    auto state = store->get_state();
    digitalWrite(LED_GREEN, state.leds.green);
    digitalWrite(LED_BLUE, state.leds.blue);
    digitalWrite(LED_RED, state.leds.red);
  }

  void setup(BotContext *context) {
    pinMode(LED_GREEN, OUTPUT);
    pinMode(LED_BLUE, OUTPUT);
    pinMode(LED_RED, OUTPUT);

    auto timer = context->timer;
    timer->set_interval(10L, output, context);
  }
}