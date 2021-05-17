#include <Arduino.h>
#include <STM32TimerInterrupt.h>
#include <STM32_ISR_Timer.h>

#include <store.hpp>
#include <timer.hpp>
#include <server.hpp>

#if !( defined(STM32F0) || defined(STM32F1) || defined(STM32F2) || defined(STM32F3)  ||defined(STM32F4) || defined(STM32F7) || \
       defined(STM32L0) || defined(STM32L1) || defined(STM32L4) || defined(STM32H7)  ||defined(STM32G0) || defined(STM32G4) || \
       defined(STM32WB) || defined(STM32MP1) )
  #error This code is designed to run on STM32F/L/H/G/WB/MP1 platform! Please check your Tools->Board setting.
#endif

// pins
// - https://github.com/stm32duino/Arduino_Core_STM32/blob/master/variants/STM32F7xx/F765Z(G-I)T_F767Z(G-I)T_F777ZIT/variant_NUCLEO_F767ZI.h

BotStore store;
BotTimer timer;
BotServer server;

volatile bool has_state_changed;

void setup()
{
  Serial.begin(115200);
  while (!Serial);

  delay(1000);
  
  timer.setup();
  server.begin();

  store.subscribe([](StateBot state) {
    has_state_changed = true;
  });

  pinMode(LED_GREEN, OUTPUT);
  pinMode(LED_BLUE, OUTPUT);
  pinMode(LED_RED, OUTPUT);

  timer.set_interval(10L, [](){
    StateBot state = store.get_state();
    digitalWrite(LED_GREEN, state.leds.green);
    digitalWrite(LED_BLUE, state.leds.blue);
    digitalWrite(LED_RED, state.leds.red);
  });

  timer.set_interval(1000L, [](){
    store.dispatch(ActionLedToggle {
      led_id: LED_ID::GREEN
    });
  });

  timer.set_interval(2000L, [](){
    store.dispatch(ActionLedToggle {
      led_id: LED_ID::BLUE
    });
  });

  timer.set_interval(4000L, [](){
    store.dispatch(ActionLedToggle {
      led_id: LED_ID::RED
    });
  });
}

void loop()
{
  if (has_state_changed) {
    StateBot state = store.get_state();

    String status = "";
    if (state.leds.green) status += ":green";
    if (state.leds.blue) status += ":blue";
    if (state.leds.red) status += ":red";

    server.events.send(status.c_str(), "status", millis());
    has_state_changed = false;
  }
}