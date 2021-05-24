#pragma once

#include <STM32TimerInterrupt.h>
#include <STM32_ISR_Timer.h>

#define HW_TIMER_INTERVAL_MS 50

STM32_ISR_Timer isr_timer;

void timer_handler() {
  isr_timer.run();
}

class BotTimer {
  public:
    STM32Timer itimer;
    
    // F767ZI can select itimer from TIM1-TIM14
    BotTimer() : itimer(TIM1) {};

    void setup() {
      Serial.print(F("\nStarting TimerInterrupt on ")); Serial.println(BOARD_NAME);
      Serial.println(STM32_TIMER_INTERRUPT_VERSION);
      Serial.print(F("CPU Frequency = ")); Serial.print(F_CPU / 1000000); Serial.println(F(" MHz"));

      // Interval in microsecs
      if (itimer.attachInterruptInterval(HW_TIMER_INTERVAL_MS * 1000, timer_handler))
      {
        Serial.print(F("Starting ITimer OK, millis() = ")); Serial.println(millis());
      }
      else {
        Serial.println(F("Can't set ITimer. Select another freq. or timer"));
      }
    }

    int16_t set_interval(uint32_t interval, timerCallback callback) {
      return isr_timer.setInterval(interval, callback);
    }

    int16_t set_interval(uint32_t interval, timerCallback_p callback, void *params) {
      return isr_timer.setInterval(interval, callback, params);
    }
};