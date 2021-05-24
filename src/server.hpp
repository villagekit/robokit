#pragma once

#include <functional>

#include <Arduino.h>
#include <LwIP.h>
#include <STM32Ethernet.h>
#include <AsyncWebServer_STM32.h>

#include <ui/index.h>

// TODO: generate random mac on first run and store in EEPROM
uint8_t mac[] = { 0x7F, 0x41, 0x26, 0xB1, 0x0E, 0xC6 };

class BotServer {
  public:
    IPAddress ip;
    AsyncWebServer web_server;
    AsyncEventSource events;

    BotServer() :
      ip(10, 0, 0, 2),
      web_server(80),
      events("/events") {};

    void begin() { 
      Serial.println("\nStart AsyncWebServer on " + String(BOARD_NAME));
      Ethernet.begin(mac, ip);

      web_server.on("/", HTTP_ANY, std::bind(&BotServer::handle_index, this, std::placeholders::_1));
      web_server.onNotFound(std::bind(&BotServer::handle_not_found, this, std::placeholders::_1));

      events.onConnect(std::bind(&BotServer::on_events_connect, this, std::placeholders::_1));
      web_server.addHandler(&events);

      web_server.begin();

      Serial.print(F("HTTP EthernetWebServer is @ IP : "));
      Serial.println(Ethernet.localIP());
    }

    void handle_index (AsyncWebServerRequest *request) {
      AsyncWebServerResponse *response = request->beginResponse(200, "text/html", PAGE_INDEX);
      request->send(response);
    }

    void handle_not_found (AsyncWebServerRequest *request) {
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

    void on_events_connect (AsyncEventSourceClient *client) {
      client->send("hello!", NULL);
    }
};