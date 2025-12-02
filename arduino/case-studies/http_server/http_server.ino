/* ESP32 HTTP IoT Server Example for Wokwi.com

  https://wokwi.com/projects/320964045035274834

  To test, you need the Wokwi IoT Gateway, as explained here:

  https://docs.wokwi.com/guides/esp32-wifi#the-private-gateway

  Then start the simulation, and open http://localhost:9080
  in another browser tab.

  Note that the IoT Gateway requires a Wokwi Club subscription.
  To purchase a Wokwi Club subscription, go to https://wokwi.com/club
*/

#define USE_EMBER // uncomment this to run the http server with ember.

#include <WiFi.h>
#include <WiFiClient.h>
#include <SPI.h>
#include <Utils.h>

#ifdef USE_EMBER
#include <Ember.h>
#include <HttpParse.h>
#include "./WebServerBehaviour.h"
#else
#include <WebServer.h>
#include <uri/UriBraces.h>
#endif // USE_EMBER

#include <memory>

#define WIFI_SSID "hofterdoele-internet-iot"
#define WIFI_PASSWORD "notthesamepasswordasmymainwifi1234;)"

#ifdef USE_EMBER
std::unique_ptr<ember::Container> container;
#else
WebServer server(80);
#endif // USE_EMBER

const int LED1 = 2;
const int LED2 = 21;

bool led1State = false;
bool led2State = false;

String format_html () {
  String response = R"(
    <!DOCTYPE html><html>
      <head>
        <title>ESP32 Web Server Demo</title>
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <style>
          html { font-family: sans-serif; text-align: center; }
          body { display: inline-flex; flex-direction: column; }
          h1 { margin-bottom: 1.2em; } 
          h2 { margin: 0; }
          div { display: grid; grid-template-columns: 1fr 1fr; grid-template-rows: auto auto; grid-auto-flow: column; grid-gap: 1em; }
          .btn { background-color: #5B5; border: none; color: #fff; padding: 0.5em 1em;
                 font-size: 2em; text-decoration: none }
          .btn.OFF { background-color: #333; }
        </style>
      </head>
            
      <body>
        <h1>ESP32 Web Server</h1>

        <div>
          <h2>LED 1</h2>
          <a href="/toggle/1" class="btn LED1_TEXT">LED1_TEXT</a>
          <h2>LED 2</h2>
          <a href="/toggle/2" class="btn LED2_TEXT">LED2_TEXT</a>
        </div>
      </body>
    </html>
  )";
  response.replace("LED1_TEXT", led1State ? "ON" : "OFF");
  response.replace("LED2_TEXT", led2State ? "ON" : "OFF");
  return response;
}

#ifdef USE_EMBER

std::tuple<std::string, uint16_t> handle_request(
  const httpparse::Request& req,
  ember::behaviour::Context<>& ctx,
  ember::Unit& state
) {
  return std::make_tuple<std::string, uint16_t>(std::string("Hello, World"), 200);
}

#endif // USE_EMBER

unsigned long current_time = micros();
unsigned int ticks = 0;

void setup(void) {
  const unsigned long start_micros = micros();

  Serial.begin(115200);
  pinMode(LED1, OUTPUT);
  pinMode(LED2, OUTPUT);

  utils::connect_wifi(WIFI_SSID, WIFI_PASSWORD, true);

  Serial.print("Peripheral setup: ");
  Serial.println(micros() - start_micros);

  #ifdef USE_EMBER

  const unsigned long ember_setup_micros = micros();

  // Initialize the embers required resources.
  ember::initialize(ember::logging::LogLevel::Debug);

  // Create the main container instance.
  container = std::make_unique<ember::Container>();

  ember::Agent<> web_server_agent("web-server-agent", ember::Unit());
  web_server_agent.add_behaviour(std::make_unique<WebServer<ember::Unit>>(handle_request, 80));

  container->add_agent(std::move(web_server_agent));

  Serial.print("Ember setup: ");
  Serial.println(micros() - ember_setup_micros);

  #else
  
  server.on("/", []() {
    server.send(200, "text/html", format_html());
  });

  server.on(UriBraces("/toggle/{}"), []() {
    String led = server.pathArg(0);
    Serial.print("Toggle LED #");
    Serial.println(led);

    switch (led.toInt()) {
      case 1:
        led1State = !led1State;
        digitalWrite(LED1, led1State);
        break;
      case 2:
        led2State = !led2State;
        digitalWrite(LED2, led2State);
        break;
    }

    server.send(200, "text/html", format_html());
  });

  server.begin();

  #endif // USE_EMBER
  Serial.println("HTTP server started");
}

void loop(void) {
  ticks += 1;
  const unsigned long new_time = micros();
  if (static_cast<float>(new_time - current_time) / 1e6 >= 1.0) {
    Serial.print("tps: ");
    Serial.println(ticks);

    ticks = 0;
    current_time = new_time;
  }

  #ifdef USE_EMBER
  
  ember::Container::PollResult result = container->poll();
  if (result.should_stop) {
    Serial.println(
      (result.status == 0) ? "Finished executing." : "Container exited with an error!"
    );
    exit(result.status);
  }

  #else

  server.handleClient();

  #endif // USE_EMBER
}
