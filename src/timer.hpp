#pragma once

#include <functional>

#include <STM32TimerInterrupt.h>
#include <STM32_ISR_Timer.h>

#define HW_TIMER_INTERVAL_MICROSECONDS 10UL * 1000UL

class BotTimer {
  public:
    STM32TimerInterrupt hardware_timer;
    STM32_ISR_Timer isr_timer;

    // F767ZI can select itimer from TIM1-TIM14
    BotTimer(TIM_TypeDef *tim) : hardware_timer(tim) {
      isr_timer.init();
      // hardware_timer = new STM32TimerInterrupt(tim);
    };

    ~BotTimer() {
      // if (hardware_timer) delete hardware_timer;
    };

    void setup() {
      if (hardware_timer.attachInterruptInterval(HW_TIMER_INTERVAL_MICROSECONDS, std::bind(&BotTimer::timer_handler, this)))
      {
        Serial.print(F("Starting Timer OK, millis() = ")); Serial.println(millis());
      }
      else {
        Serial.println(F("Can't set Timer. Select another freq. or timer"));
      }
    }

    int16_t set_interval(uint32_t interval, timerCallback callback) {
      return isr_timer.setInterval(interval, callback);
    }

    int16_t set_interval(uint32_t interval, timerCallback_p callback, void *params) {
      return isr_timer.setInterval(interval, callback, params);
    }

  private:
    void timer_handler() {
      isr_timer.run();
    }
};