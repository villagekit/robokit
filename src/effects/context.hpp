#pragma once

#include <STM32_ISR_Timer.h>

#include <store.hpp>

struct BotContext {
  BotStore *store;
  STM32_ISR_Timer *isr_timer;
};