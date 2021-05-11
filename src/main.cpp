#include <Arduino.h>
#include "STM32TimerInterrupt.h"
#include "STM32_ISR_Timer.h"
#include <store.hpp>
#include <mapbox/variant.hpp>
#include <mapbox/variant_visitor.hpp>

#if !( defined(STM32F0) || defined(STM32F1) || defined(STM32F2) || defined(STM32F3)  ||defined(STM32F4) || defined(STM32F7) || \
       defined(STM32L0) || defined(STM32L1) || defined(STM32L4) || defined(STM32H7)  ||defined(STM32G0) || defined(STM32G4) || \
       defined(STM32WB) || defined(STM32MP1) )
  #error This code is designed to run on STM32F/L/H/G/WB/MP1 platform! Please check your Tools->Board setting.
#endif

#define TIMER_INTERVAL_MS         100
#define HW_TIMER_INTERVAL_MS      50

// F767ZI can select Timer from TIM1-TIM14
STM32Timer ITimer(TIM1);

// Each STM32_ISR_Timer can service 16 different ISR-based timers
STM32_ISR_Timer ISR_Timer;

#define TIMER_INTERVAL_TICK           100L

struct StateLeds {
  bool green = true;
  bool blue = true;
  bool red = true;
};

enum class LED_ID { GREEN, RED, BLUE };
struct ActionLedToggle {
  LED_ID led_id;
};
struct ActionLedOther {};
using ActionLeds = mapbox::util::variant<ActionLedToggle, ActionLedOther>;

StateLeds reducer_leds(StateLeds state, ActionLeds action) {
  action.match(
    [&state](const ActionLedToggle action) {
      switch (action.led_id) {
        case LED_ID::GREEN:
          state.green = !state.green;
          break;
        case LED_ID::BLUE:
          state.blue = !state.blue;
          break;
        case LED_ID::RED:
          state.red = !state.red;
          break;
      }
    },
    [](ActionLedOther) {}
  );
  
  return state;
}

struct ActionClockTick {};
using ActionClock = mapbox::util::variant<ActionClockTick>;

struct StateClock {
  uint16_t ticks = 0;
};

StateClock reducer_clock(StateClock state, ActionClock action) {
  action.match(
    [&state](const ActionClockTick) {
      state.ticks++;
    }
  );

  return state;
}

struct StateBot {
  StateLeds leds;
  StateClock clock;
};

using ActionBot = mapbox::util::variant<ActionLeds, ActionClock>;

StateBot reducer(StateBot state, ActionBot action) {
  action.match(
    [&state](const ActionLeds a) {
      state.leds = reducer_leds(state.leds, a);
    },
    [&state](const ActionClock a) {
      state.clock = reducer_clock(state.clock, a);
    }
  );

  return state;
}

void TimerHandler()
{
  ISR_Timer.run();
}

mapbox::util::variant<int> singleton = 5;

void setup()
{
  Serial.begin(115200);
  while (!Serial);

  delay(100);

  Serial.print(F("\nStarting TimerInterruptLEDDemo on ")); Serial.println(BOARD_NAME);
  Serial.println(STM32_TIMER_INTERRUPT_VERSION);
  Serial.print(F("CPU Frequency = ")); Serial.print(F_CPU / 1000000); Serial.println(F(" MHz"));

  // Interval in microsecs
  if (ITimer.attachInterruptInterval(HW_TIMER_INTERVAL_MS * 1000, TimerHandler))
  {
    Serial.print(F("Starting ITimer OK, millis() = ")); Serial.println(millis());
  }
  else {
    Serial.println(F("Can't set ITimer. Select another freq. or timer"));
  }

  /*
  green_machine.setup();
  blue_machine.setup();
  red_machine.setup();

  ISR_Timer.setInterval(TIMER_INTERVAL_TICK, [](){ green_machine.tick(); });
  ISR_Timer.setInterval(TIMER_INTERVAL_TICK, [](){ blue_machine.tick(); });
  ISR_Timer.setInterval(TIMER_INTERVAL_TICK, [](){ red_machine.tick(); });
  */
}

void loop()
{
  /* Nothing to do all is done by hardware. Even no interrupt required. */
}