#include <Arduino.h>
#include <variant>

#include "STM32TimerInterrupt.h"
#include "STM32_ISR_Timer.h"

#include <overloaded.hpp>
#include <store.hpp>

#if !( defined(STM32F0) || defined(STM32F1) || defined(STM32F2) || defined(STM32F3)  ||defined(STM32F4) || defined(STM32F7) || \
       defined(STM32L0) || defined(STM32L1) || defined(STM32L4) || defined(STM32H7)  ||defined(STM32G0) || defined(STM32G4) || \
       defined(STM32WB) || defined(STM32MP1) )
  #error This code is designed to run on STM32F/L/H/G/WB/MP1 platform! Please check your Tools->Board setting.
#endif

// pins
// - https://github.com/stm32duino/Arduino_Core_STM32/blob/master/variants/STM32F7xx/F765Z(G-I)T_F767Z(G-I)T_F777ZIT/variant_NUCLEO_F767ZI.h

#define TIMER_INTERVAL_MS         100
#define HW_TIMER_INTERVAL_MS      50

// F767ZI can select Timer from TIM1-TIM14
STM32Timer ITimer(TIM1);

STM32_ISR_Timer ISR_Timer;

struct StateLeds {
  bool green = true;
  bool blue = true;
  bool red = true;
};

enum class LED_ID { GREEN, RED, BLUE };
struct ActionLedToggle {
  LED_ID led_id;
};
using ActionLeds = std::variant<ActionLedToggle>;

StateLeds reducer_leds(StateLeds state, ActionLeds action) {
  std::visit(overloaded {
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
    }
  }, action);
  
  return state;
}

struct ActionClockTick {};
using ActionClock = std::variant<ActionClockTick>;

struct StateClock {
  uint16_t ticks = 0;
};

StateClock reducer_clock(StateClock state, ActionClock action) {
  std::visit(overloaded {
    [&state](const ActionClockTick) {
      state.ticks++;
    }
  }, action);

  return state;
}

struct StateBot {
  StateLeds leds = StateLeds {};
  StateClock clock = StateClock {};
};

using ActionBot = std::variant<ActionLeds, ActionClock>;

StateBot reducer_bot(StateBot state, ActionBot action) {
  noInterrupts();

  std::visit(overloaded {
    [&state](const ActionLeds a) {
      state.leds = reducer_leds(state.leds, a);
    },
    [&state](const ActionClock a) {
      state.clock = reducer_clock(state.clock, a);
    }
  }, action);

  interrupts();

  return state;
}

Store<StateBot, ActionBot> store(reducer_bot, StateBot {});

void TimerHandler()
{
  ISR_Timer.run();
}

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

  pinMode(LED_GREEN, OUTPUT);
  pinMode(LED_BLUE, OUTPUT);
  pinMode(LED_RED, OUTPUT);

  ISR_Timer.setInterval(100L, [](){
    StateBot state = store.getState();
    digitalWrite(LED_GREEN, state.leds.green);
    digitalWrite(LED_BLUE, state.leds.blue);
    digitalWrite(LED_RED, state.leds.red);
  });

  ISR_Timer.setInterval(400L, [](){
    store.dispatch(ActionLedToggle {
      led_id: LED_ID::GREEN
    });
  });

  ISR_Timer.setInterval(800L, [](){
    store.dispatch(ActionLedToggle {
      led_id: LED_ID::BLUE
    });
  });

  ISR_Timer.setInterval(1600L, [](){
    store.dispatch(ActionLedToggle {
      led_id: LED_ID::RED
    });
  });
}

void loop()
{
  /* Nothing to do all is done by hardware. Even no interrupt required. */
}