#pragma once

#include <effects/context.hpp>
#include <effects/clock.hpp>
#include <effects/leds.hpp>
#include <effects/motors.hpp>

#include <STM32TimerInterrupt.h>
#include <STM32_ISR_Timer.h>

namespace BotEffects {
  const unsigned long HW_TIMER_INTERVAL_MICROSECONDS = 1000UL;

  // F767ZI can select itimer from TIM1-TIM14
  STM32Timer hw_timer = STM32Timer(TIM1);
  STM32_ISR_Timer isr_timer;

  BotContext context;

  void timer_handler() {
    isr_timer.run();
  }

  void setup_timer() {
    if (!hw_timer.attachInterruptInterval(HW_TIMER_INTERVAL_MICROSECONDS, timer_handler)) {
      Serial.println(F("Failure to start bot timer!"));
    }
  }

  void setup(BotStore *store) {
    setup_timer();

    context = BotContext {
      store,
      &isr_timer
    };

    LedsEffects::setup(&context);
    ClockEffects::setup(&context);
    MotorsEffects::setup(&context);
  }
}