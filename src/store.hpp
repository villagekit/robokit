#pragma once

#include <vector>

#include <redux.hpp>
#include <RingBufCPP.h>
#include <SimplyAtomic.h>

#include <models/bot.hpp>

class BotStore {
	typedef std::function<void(BotModel::State)> Subscriber;

  private:
    Store<BotModel::State, BotModel::Action> store;
    RingBufCPP<BotModel::Action, 20 * sizeof(BotModel::Action)> queued_actions;
    RingBufCPP<BotModel::Action, 20 * sizeof(BotModel::Action)> processing_actions;
    std::vector<Subscriber> subscribers;

    void notify() {
      auto state = get_state();

      for (size_t i = 0; i < subscribers.size(); i++) {
        subscribers[i](state);
      }
    }

  public:
    BotStore(): store(BotModel::reducer, BotModel::State {}), queued_actions(), processing_actions() {};

    BotModel::State get_state() {
      return store.get_state();
    }

    void dispatch(BotModel::Action action) {
      // if ring buffer is full, infinite loop (which will lapse watchdog timer)
      if (queued_actions.isFull()) {
        Serial.println("ERROR: action queue is full!");
        while (true) {};
      }

      // queue action to ring buffer.
      //   we do a special dance so all actions are processed safely
      //   in the main loop, even if dispatched in an interrupt.
      queued_actions.add(action);
    }

    void subscribe(Subscriber subscriber) {
      subscribers.push_back(std::move(subscriber));
    }

    void loop() {    
      // copy queued actions to processing buffer
      ATOMIC() {
        for (size_t i = 0; i < queued_actions.numElements(); i++) {
          BotModel::Action queued_action;
          queued_actions.pull(&queued_action);
          processing_actions.add(queued_action);
        }
      };

      bool has_state_changed = !processing_actions.isEmpty();
      for (size_t i = 0; i < processing_actions.numElements(); i++) {
        BotModel::Action processing_action;
        processing_actions.pull(&processing_action);
        store.dispatch(processing_action);
      }

      // if state has changed, notify subscribers
      if (has_state_changed) {
        notify();
      }
    }
};