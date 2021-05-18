#pragma once

#include <redux.hpp>

#include <models/bot.hpp>

class BotStore : public Store<BotModel::State, BotModel::Action> {
  public:
    BotStore(): Store(BotModel::reducer, BotModel::State {}) {};
};