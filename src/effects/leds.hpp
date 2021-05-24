#pragma once

#include <Arduino.h>

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

  void green_toggle(void *params) {
    BotContext *context = (BotContext*) params;
    auto store = context->store;
    store->dispatch(LedsModel::ActionToggle {
      led_id: LedsModel::LED_ID::GREEN
    });
  }

  void blue_toggle(void *params) {
    BotContext *context = (BotContext*) params;
    auto store = context->store;
    store->dispatch(LedsModel::ActionToggle {
      led_id: LedsModel::LED_ID::BLUE
    });
  }

  void red_toggle(void *params) {
    BotContext *context = (BotContext*) params;
    auto store = context->store;
    store->dispatch(LedsModel::ActionToggle {
      led_id: LedsModel::LED_ID::RED
    });
  }

  void setup(BotContext *context) {
    pinMode(LED_GREEN, OUTPUT);
    pinMode(LED_BLUE, OUTPUT);
    pinMode(LED_RED, OUTPUT);

    auto timer = context->timer;
    timer->set_interval(10L, output, context);
    timer->set_interval(1000L, green_toggle, context);
    timer->set_interval(2000L, blue_toggle, context);
    timer->set_interval(4000L, red_toggle, context);
  }
}