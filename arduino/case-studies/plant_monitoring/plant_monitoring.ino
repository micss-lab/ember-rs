#define USE_EMBER // Uncomment this to use the ember library.

/******************************************
  Pin Definitions
******************************************/

#define LDR_PIN 34
#define POTENTIOMETER_PIN 32
#define WATER_PUMP_PIN 27
#define LIGHT_ALERT_PIN 16
#define USER_SWITCH_PIN 33
#define LCD_SWITCH_PIN 35
#define BUZZER_PIN 3

#include "./common.h"

#ifdef USE_EMBER
#include "./agents/ControlAgent.h"
#include "./agents/LightAgent.h"
#include "./agents/MoistureAgent.h"
#include "./agents/PumpAgent.h"
#include "./agents/TempAndHumidityAgent.h"
#else
#include <DHT.h>
#define DHTPIN 14
#define DHTTYPE DHT22
DHT dht(DHTPIN, DHTTYPE);
#include "./without_ember.h"
#endif // USE_EMBER

#ifdef USE_EMBER
std::unique_ptr<ember::Container> container;
#endif // USE_EMBER

unsigned long current_time = micros();
unsigned int ticks = 0;

/******************************************
  Setup
******************************************/
void setup() {
  heap_caps_check_integrity_all(true);

  const unsigned long start_micros = micros();

  Serial.begin(115200);

  analogSetAttenuation(ADC_11db);

  pinMode(WATER_PUMP_PIN, OUTPUT);
  pinMode(LIGHT_ALERT_PIN, OUTPUT);
  pinMode(USER_SWITCH_PIN, INPUT_PULLUP);
  pinMode(LCD_SWITCH_PIN, INPUT_PULLUP);

  digitalWrite(WATER_PUMP_PIN, LOW);
  digitalWrite(LIGHT_ALERT_PIN, LOW);

  Serial.print("Peripheral setup: ");
  Serial.println(micros() - start_micros);

  #ifdef USE_EMBER

  const unsigned long ember_setup_micros = micros();

  // Initialize the embers required resources.
  ember::initialize(ember::logging::LogLevel::Debug);

  // Create the main container instance.
  container = std::make_unique<ember::Container>();

  auto control_agent = agents::control::create_control_agent(USER_SWITCH_PIN);
  auto light_agent = agents::light::create_light_agent(LDR_PIN, LIGHT_ALERT_PIN);
  auto moisture_agent = agents::moisture::create_moisture_agent(POTENTIOMETER_PIN);
  auto pump_agent = agents::pump::create_pump_agent(WATER_PUMP_PIN);
  auto temp_and_humidity_agent = agents::temp_and_humidity::create_temp_and_humidity_agent();

  container->add_agent(std::move(control_agent));
  container->add_agent(std::move(light_agent));
  container->add_agent(std::move(moisture_agent));
  container->add_agent(std::move(pump_agent));
  container->add_agent(std::move(temp_and_humidity_agent));

  Serial.print("Ember setup: ");
  Serial.println(micros() - ember_setup_micros);

  #else

  dht.begin();

  #endif // USE_EMBER

  Serial.println("Smart Plant Monitoring System Initialized");
}

/******************************************
  Loop
******************************************/
void loop() {
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

  float temperature = dht.readTemperature();
  float humidity = dht.readHumidity();

  int rawLight = analogRead(LDR_PIN);
  float sensorLux = ((4095 - rawLight) / 4095.0) * (MAX_LUX - MIN_LUX) + MIN_LUX;
  int mappedLuxGauge = (int)(((sensorLux - MIN_LUX) / (MAX_LUX - MIN_LUX)) * 4095);

  int rawMoisture = analogRead(POTENTIOMETER_PIN);
  int mappedMoistureLevel = map(rawMoisture, 0, 4095, 0, 100);

  static unsigned long lastPrintTime = 0;
  if (millis() - lastPrintTime >= 1000) {
    printSensorValues(temperature, humidity, mappedLuxGauge, rawMoisture);
    lastPrintTime = millis();
  }

  handleLightAlert(mappedLuxGauge);
  handlePumpControl(digitalRead(USER_SWITCH_PIN) == HIGH, rawMoisture);
  checkTemperatureAlert(temperature);
  checkHumidityNotification(humidity);
  checkLightNotification(mappedLuxGauge);
  checkMoistureNotification(mappedMoistureLevel);

  #endif // USE_EMBER
}

