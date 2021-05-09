#include <Arduino.h>
#include <variant> // std::variant
#include <optional> // std::optional

#if !( defined(STM32F0) || defined(STM32F1) || defined(STM32F2) || defined(STM32F3)  ||defined(STM32F4) || defined(STM32F7) || \
       defined(STM32L0) || defined(STM32L1) || defined(STM32L4) || defined(STM32H7)  ||defined(STM32G0) || defined(STM32G4) || \
       defined(STM32WB) || defined(STM32MP1) )
  #error This code is designed to run on STM32F/L/H/G/WB/MP1 platform! Please check your Tools->Board setting.
#endif

// These define's must be placed at the beginning before #include "STM32TimerInterrupt.h"
// _TIMERINTERRUPT_LOGLEVEL_ from 0 to 4
// Don't define _TIMERINTERRUPT_LOGLEVEL_ > 0. Only for special ISR debugging only. Can hang the system.
// Don't define TIMER_INTERRUPT_DEBUG > 2. Only for special ISR debugging only. Can hang the system.
#define TIMER_INTERRUPT_DEBUG         0
#define _TIMERINTERRUPT_LOGLEVEL_     0

#include "STM32TimerInterrupt.h"

#ifndef LED_BUILTIN
  #define LED_BUILTIN       PB0               // Pin 33/PB0 control on-board LED_GREEN on F767ZI
#endif

#ifndef LED_BLUE
  #define LED_BLUE          PB7               // Pin 73/PB7 control on-board LED_BLUE on F767ZI
#endif

#ifndef LED_RED
  #define LED_RED           PB14              // Pin 74/PB14 control on-board LED_BLUE on F767ZI
#endif
   
#include "STM32TimerInterrupt.h"
#include "STM32_ISR_Timer.h"

#define TIMER_INTERVAL_MS         100
#define HW_TIMER_INTERVAL_MS      50

// F767ZI can select Timer from TIM1-TIM14
STM32Timer ITimer(TIM1);

// Each STM32_ISR_Timer can service 16 different ISR-based timers
STM32_ISR_Timer ISR_Timer;

#define TIMER_INTERVAL_TICK           100L

// Events
//
struct EventStart {};
struct EventStop {};
struct EventTick {};

using Event = std::variant<EventStart, EventStop, EventTick>;

// States
//
struct Idle {};
struct Active {
  bool isOn = true;
  uint16_t ticksUntilSwitch = 20;
};

using State = std::variant<Idle, Active>;

// Transitions
//
struct Transitions {
  std::optional<State> operator()(Idle &, const EventStart &) {
    pinMode(LED_BUILTIN, OUTPUT);

    return Active{};
  }

  std::optional<State> operator()(Active &, const EventStop &) {
    return Idle{};
  }

  std::optional<State> operator()(Active &s, const EventTick &) {
    digitalWrite(LED_BUILTIN, s.isOn);

    if (s.ticksUntilSwitch-- == 0) {
      s.isOn = !s.isOn;
      s.ticksUntilSwitch = 20;
    }

    return std::nullopt;
  }

  template <typename State_t, typename Event_t>
  std::optional<State> operator()(State_t &, const Event_t &) const {
      return std::nullopt;
  }
};

template <typename Parameters, typename StateVariant, typename EventVariant, typename Transitions>
struct Machine {
  Machine(Parameters parameters);
  ~Machine();

  Parameters parameters;
  StateVariant current_state;

  void dispatch(const EventVariant &Event) {
    std::optional<StateVariant> new_state = std::visit(Transitions{}, current_state, Event);
    if (new_state)
        current_state = *std::move(new_state);
  }
};

struct Params {
  uint8_t led_pin;
  uint16_t ticks_until_switch;
};

Params green_params = {
  LED_BUILTIN, // led_pin
  10 // ticks_until_switch
};

volatile Machine<Params, State, Event, Transitions> green_machine(green_params);

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

  green_machine.dispatch(EventStart{});
  ISR_Timer.setInterval(TIMER_INTERVAL_TICK, [](){ green_machine.dispatch(EventTick{}); });
}

void loop()
{
  /* Nothing to do all is done by hardware. Even no interrupt required. */
}