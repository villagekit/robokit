#include <SimplyAtomic.h>
#include <RingBufCPP.h>
#define RB_ATOMIC_START ATOMIC() {
#define RB_ATOMIC_END }

#include <Arduino.h>
#include <IWatchdog.h>

#include <store.hpp>
#include <server.hpp>
#include <effects/bot.hpp>

#if !(defined(STM32F7))
  #error This code is designed to run on STM32F7 platform!
#endif

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

  // initialize watchdog with 2 millisecond timeout
  IWatchdog.begin(2000UL);
}

void loop()
{
  store.loop();
  
  // reset watchdog timeout
  IWatchdog.reload();
}