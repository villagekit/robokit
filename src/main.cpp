#include <Arduino.h>
#include <LwIP.h>
#include <STM32Ethernet.h>
#include <AsyncWebServer_STM32.h>
#include <STM32TimerInterrupt.h>
#include <STM32_ISR_Timer.h>

#include <mpark/variant.hpp>
#include <overload.hpp>
#include <store.hpp>

#include <ui/index.h>

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
using ActionLeds = mpark::variant<ActionLedToggle>;

StateLeds reducer_leds(StateLeds state, ActionLeds action) {
  mpark::visit(overload(
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
  ), action);
  
  return state;
}

struct ActionClockTick {};
using ActionClock = mpark::variant<ActionClockTick>;

struct StateClock {
  uint16_t ticks = 0;
};

StateClock reducer_clock(StateClock state, ActionClock action) {
  mpark::visit(overload(
    [&state](const ActionClockTick) {
      state.ticks++;
    }
  ), action);

  return state;
}

struct StateBot {
  StateLeds leds = StateLeds {};
  StateClock clock = StateClock {};
};

using ActionBot = mpark::variant<ActionLeds, ActionClock>;

StateBot reducer_bot(StateBot state, ActionBot action) {
  noInterrupts();

  mpark::visit(overload(
    [&state](const ActionLeds a) {
      state.leds = reducer_leds(state.leds, a);
    },
    [&state](const ActionClock a) {
      state.clock = reducer_clock(state.clock, a);
    }
  ), action);

  interrupts();

  return state;
}

Store<StateBot, ActionBot> store(reducer_bot, StateBot {});

IPAddress ip(10, 0, 0, 2);
AsyncWebServer server(80);
// uint8_t mac[] = { MAC_ADDR0, MAC_ADDR1, MAC_ADDR2, MAC_ADDR3, MAC_ADDR4, MAC_ADDR5 };
uint8_t mac[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0x32, 0x01 };
AsyncEventSource events("/events");

void handleNotFound(AsyncWebServerRequest *request)
{
  String message = "File Not Found\n\n";

  message += "URI: ";
  //message += server.uri();
  message += request->url();
  message += "\nMethod: ";
  message += (request->method() == HTTP_GET) ? "GET" : "POST";
  message += "\nArguments: ";
  message += request->args();
  message += "\n";

  for (uint8_t i = 0; i < request->args(); i++)
  {
    message += " " + request->argName(i) + ": " + request->arg(i) + "\n";
  }

  request->send(404, "text/plain", message);
}

void TimerHandler()
{
  ISR_Timer.run();
}

volatile bool has_state_changed;

void setup()
{
  Serial.begin(115200);
  while (!Serial);

  delay(1000);

  Serial.print(F("\nStarting TimerInterrupt on ")); Serial.println(BOARD_NAME);
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

  Serial.println("\nStart AsyncWebServer on " + String(BOARD_NAME));
  Ethernet.begin(mac, ip);

  server.on("/", HTTP_GET, [](AsyncWebServerRequest * request)
  {
    AsyncWebServerResponse *response = request->beginResponse(200, "text/html", PAGE_INDEX);
    request->send(response);
  });

  events.onConnect([](AsyncEventSourceClient * client) 
  {
    client->send("hello!", NULL);
  });

  has_state_changed = false;
  store.subscribe([](StateBot state){
    has_state_changed = true;
  });
  
  server.addHandler(&events);
  server.onNotFound(handleNotFound);
  server.begin();

  Serial.print(F("HTTP EthernetWebServer is @ IP : "));
  Serial.println(Ethernet.localIP());

  pinMode(LED_GREEN, OUTPUT);
  pinMode(LED_BLUE, OUTPUT);
  pinMode(LED_RED, OUTPUT);

  ISR_Timer.setInterval(10L, [](){
    StateBot state = store.getState();
    digitalWrite(LED_GREEN, state.leds.green);
    digitalWrite(LED_BLUE, state.leds.blue);
    digitalWrite(LED_RED, state.leds.red);
  });

  ISR_Timer.setInterval(1000L, [](){
    store.dispatch(ActionLedToggle {
      led_id: LED_ID::GREEN
    });
  });

  ISR_Timer.setInterval(2000L, [](){
    store.dispatch(ActionLedToggle {
      led_id: LED_ID::BLUE
    });
  });

  ISR_Timer.setInterval(4000L, [](){
    store.dispatch(ActionLedToggle {
      led_id: LED_ID::RED
    });
  });
}

void loop()
{
  if (has_state_changed) {
    StateBot state = store.getState();

    String status = "";
    if (state.leds.green) status += ":green";
    if (state.leds.blue) status += ":blue";
    if (state.leds.red) status += ":red";

    events.send(status.c_str(), "status", millis());
    has_state_changed = false;
  }
}