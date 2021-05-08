#include <Arduino.h>

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

static uint16_t GREEN_TICKS_TOTAL = 10;
static uint16_t BLUE_TICKS_TOTAL = 20;
static uint16_t RED_TICKS_TOTAL = 40;

volatile bool greenStatus = false;
volatile bool blueStatus = false;
volatile bool redStatus = false;

volatile uint16_t greenTicksLeft = 0;
volatile uint16_t blueTicksLeft = 0;
volatile uint16_t redTicksLeft = 0;

void TimerHandler()
{
  ISR_Timer.run();
}

void processGreen()
{
  if (greenTicksLeft == 0) {
    greenStatus = !greenStatus;
    greenTicksLeft = GREEN_TICKS_TOTAL;
  }

  digitalWrite(LED_BUILTIN, greenStatus);

  greenTicksLeft -= 1;
}

void processBlue()
{
  if (blueTicksLeft == 0) {
    blueStatus = !blueStatus;
    blueTicksLeft = BLUE_TICKS_TOTAL;
  }

  digitalWrite(LED_BLUE, blueStatus);

  blueTicksLeft -= 1;
}

void processRed()
{
  if (redTicksLeft == 0) {
    redStatus = !redStatus;
    redTicksLeft = RED_TICKS_TOTAL;
  }

  digitalWrite(LED_RED, redStatus);

  redTicksLeft -= 1;
}

void setup()
{
  Serial.begin(115200);
  while (!Serial);

  delay(100);

  Serial.print(F("\nStarting TimerInterruptLEDDemo on ")); Serial.println(BOARD_NAME);
  Serial.println(STM32_TIMER_INTERRUPT_VERSION);
  Serial.print(F("CPU Frequency = ")); Serial.print(F_CPU / 1000000); Serial.println(F(" MHz"));

  // configure pin in output mode
  pinMode(LED_BUILTIN, OUTPUT);
  pinMode(LED_BLUE, OUTPUT);
  pinMode(LED_RED, OUTPUT);

  // Interval in microsecs
  if (ITimer.attachInterruptInterval(HW_TIMER_INTERVAL_MS * 1000, TimerHandler))
  {
    Serial.print(F("Starting ITimer OK, millis() = ")); Serial.println(millis());
  }
  else {
    Serial.println(F("Can't set ITimer. Select another freq. or timer"));
  }

  // You can use up to 16 timer for each ISR_Timer
  ISR_Timer.setInterval(TIMER_INTERVAL_TICK, processGreen);
  ISR_Timer.setInterval(TIMER_INTERVAL_TICK, processBlue);
  ISR_Timer.setInterval(TIMER_INTERVAL_TICK, processRed);
}


void loop()
{
  /* Nothing to do all is done by hardware. Even no interrupt required. */
}