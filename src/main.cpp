#include <Arduino.h>

#include <store.hpp>
#include <server.hpp>
#include <effects/bot.hpp>

#if !( defined(STM32F0) || defined(STM32F1) || defined(STM32F2) || defined(STM32F3)  ||defined(STM32F4) || defined(STM32F7) || \
       defined(STM32L0) || defined(STM32L1) || defined(STM32L4) || defined(STM32H7)  ||defined(STM32G0) || defined(STM32G4) || \
       defined(STM32WB) || defined(STM32MP1) )
  #error This code is designed to run on STM32F/L/H/G/WB/MP1 platform! Please check your Tools->Board setting.
#endif

#include <SimplyAtomic.h>
#define RB_ATOMIC_START ATOMIC() {
#define RB_ATOMIC_END }

// pins
// - https://github.com/stm32duino/Arduino_Core_STM32/blob/master/variants/STM32F7xx/F765Z(G-I)T_F767Z(G-I)T_F777ZIT/variant_NUCLEO_F767ZI.h

BotServer server;
BotStore store;

void setup()
{
  Serial.begin(115200);
  while (!Serial);

  delay(1000);

  Serial.println();
  Serial.print(F("Starting GridBot on ")); Serial.println(BOARD_NAME);
  Serial.print(F("CPU Frequency = ")); Serial.print(F_CPU / 1000000); Serial.println(F(" MHz"));
  Serial.println();

  server.begin(&store);
  BotEffects::setup(&store);
}

void loop()
{
  store.loop();
  delay(5);
}