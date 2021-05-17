#include <redux.hpp>

#include <models/bot.hpp>

class BotStore : public Store<StateBot, ActionBot> {
  public:
    BotStore(): Store(reducer_bot, StateBot {}) {};
};