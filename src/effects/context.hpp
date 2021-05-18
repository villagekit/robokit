#pragma once

#include <store.hpp>
#include <timer.hpp>

struct BotContext {
  BotStore *store;
  BotTimer *timer;
};