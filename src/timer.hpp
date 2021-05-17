#include <STM32TimerInterrupt.h>
#include <STM32_ISR_Timer.h>

#define TIMER_INTERVAL_MS 100
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

    bool set_interval(unsigned long interval, timerCallback callback) {
      return isr_timer.setInterval(interval, callback);
    }
};