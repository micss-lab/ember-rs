//! Based on the following template:
//!
//! /******************************************
//!   Blynk Template Setup
//! ******************************************/
//! #define BLYNK_TEMPLATE_ID    "TMPL5_-eK22-6"
//! #define BLYNK_TEMPLATE_NAME  "Smart Plant Monitoring System"
//! #define BLYNK_AUTH_TOKEN     "X_-ViewHXALPXt1Zpjb8uL0_q1QqfO83"
//!
//! char ssid[] = "Wokwi-GUEST";
//! char pass[] = "";
//!
//! /******************************************
//!   Firebase Configuration
//! ******************************************/
//! #define FIREBASE_HOST "smart-plant-monitoring-s-4249f-default-rtdb.firebaseio.com/"
//! #define FIREBASE_SECRET "rrmEA8xd0R3oniQ1J0MPRqfvYqttf4ggYRYp9b7T"
//!
//! /******************************************
//!   Include Libraries
//! ******************************************/
//! #include <WiFi.h>
//! #include <BlynkSimpleEsp32.h>
//! #include <DHT.h>
//! #include <LiquidCrystal_I2C.h>
//! #include <FirebaseESP32.h>
//! #include "addons/TokenHelper.h"
//! #include "addons/RTDBHelper.h"
//!
//! /******************************************
//!   Pin Definitions
//! ******************************************/
//! #define DHTPIN 14
//! #define DHTTYPE DHT22
//! #define LDR_PIN 34
//! #define POTENTIOMETER_PIN 32
//! #define WATER_PUMP_PIN 27
//! #define LIGHT_ALERT_PIN 16
//! #define BUZZER_PIN 19
//! #define USER_SWITCH_PIN 33
//! #define LCD_SWITCH_PIN 35
//!
//! /******************************************
//!   Thresholds & Calibration
//! ******************************************/
//! #define LIGHT_THRESHOLD       2000
//! #define MOISTURE_THRESHOLD    2000
//! #define TEMP_HIGH_THRESHOLD   36.0
//! #define TEMP_LOW_THRESHOLD    -18.0
//! const float MIN_LUX = 0.1;
//! const float MAX_LUX = 100000.0;
//!
//! /******************************************
//!   State Flags & Control
//! ******************************************/
//! bool pumpActive = false;
//! unsigned long pumpStartTime = 0;
//! bool lcdActive = false;
//! unsigned long lcdStartTime = 0;
//! const unsigned long LCD_DURATION = 25000;
//!
//! bool highNotified = false;
//! bool lowNotified = false;
//! bool highHumidityNotified = false;
//! bool lowHumidityNotified = false;
//! bool lowLightNotified = false;
//! bool highLightNotified = false;
//! bool virtualLCD = false;
//! bool virtualPump = false;
//! bool lowMoisture = false;
//! bool highMoisture = false;
//!
//! /******************************************
//!   Firebase & Sensor Objects
//! ******************************************/
//! DHT dht(DHTPIN, DHTTYPE);
//! LiquidCrystal_I2C lcd(0x27, 20, 4);
//! FirebaseData fbdo;
//! FirebaseAuth auth;
//! FirebaseConfig config;
//!
//! /******************************************
//!   Blynk Handlers
//! ******************************************/
//! BLYNK_WRITE(V6) {
//!   virtualLCD = (param.asInt() == 1);
//! }
//!
//! BLYNK_WRITE(V7) {
//!   virtualPump = (param.asInt() == 1);
//! }
//!
//! /******************************************
//!   Setup
//! ******************************************/
//! void setup() {
//!   Serial.begin(115200);
//!   Blynk.begin(BLYNK_AUTH_TOKEN, ssid, pass);
//!   config.database_url = FIREBASE_HOST;
//!   config.signer.tokens.legacy_token = FIREBASE_SECRET;
//!   Firebase.begin(&config, &auth);
//!   Firebase.reconnectWiFi(true);
//!
//!   dht.begin();
//!   analogSetAttenuation(ADC_11db);
//!   lcd.init();
//!   lcd.noBacklight();
//!   lcd.clear();
//!
//!   pinMode(WATER_PUMP_PIN, OUTPUT);
//!   pinMode(LIGHT_ALERT_PIN, OUTPUT);
//!   pinMode(BUZZER_PIN, OUTPUT);
//!   pinMode(USER_SWITCH_PIN, INPUT_PULLUP);
//!   pinMode(LCD_SWITCH_PIN, INPUT_PULLUP);
//!
//!   digitalWrite(WATER_PUMP_PIN, LOW);
//!   digitalWrite(LIGHT_ALERT_PIN, LOW);
//!   noTone(BUZZER_PIN);
//!
//!   Serial.println("Smart Plant Monitoring System Initialized");
//! }
//!
//! /******************************************
//!   Loop
//! ******************************************/
//! void loop() {
//!   Blynk.run();
//!   bool physicalLCD = (digitalRead(LCD_SWITCH_PIN) == LOW);
//!   bool physicalPump = (digitalRead(USER_SWITCH_PIN) == LOW);
//!   bool effectiveLCDSwitch = virtualLCD || physicalLCD;
//!   bool effectivePumpSwitch = virtualPump || physicalPump;
//!
//!   handleLCDControl(effectiveLCDSwitch);
//!
//!   float temperature = dht.readTemperature();
//!   float humidity = dht.readHumidity();
//!
//!   int rawLight = analogRead(LDR_PIN);
//!   float sensorLux = ((4095 - rawLight) / 4095.0) * (MAX_LUX - MIN_LUX) + MIN_LUX;
//!   int mappedLuxGauge = (int)(((sensorLux - MIN_LUX) / (MAX_LUX - MIN_LUX)) * 4095);
//!
//!   int rawMoisture = analogRead(POTENTIOMETER_PIN);
//!   int mappedMoistureLevel = map(rawMoisture, 0, 4095, 0, 100);
//!
//!   static unsigned long lastPrintTime = 0;
//!   if (millis() - lastPrintTime >= 1000) {
//!     printSensorValues(temperature, humidity, mappedLuxGauge, rawMoisture);
//!     lastPrintTime = millis();
//!   }
//!
//!   if (lcdActive) updateLcd(temperature, humidity, mappedLuxGauge, rawMoisture);
//!
//!   handleLightAlert(mappedLuxGauge);
//!   handlePumpControl(effectivePumpSwitch, rawMoisture);
//!   checkTemperatureAlert(temperature);
//!   checkHumidityNotification(humidity);
//!   checkLightNotification(mappedLuxGauge);
//!   checkMoistureNotification(mappedMoistureLevel);
//!
//!   Blynk.virtualWrite(V0, temperature);
//!   Blynk.virtualWrite(V1, humidity);
//!   Blynk.virtualWrite(V2, mappedLuxGauge);
//!   Blynk.virtualWrite(V3, mappedMoistureLevel);
//!   Blynk.virtualWrite(V4, pumpActive ? 1 : 0);
//!   Blynk.virtualWrite(V5, (mappedLuxGauge < LIGHT_THRESHOLD) ? mappedLuxGauge : 0);
//!   Blynk.virtualWrite(V9, digitalRead(USER_SWITCH_PIN));
//!
//!   // ✅ Create a new log entry for each set of sensor values
//!   String logPath = "/logs/" + String(millis());
//!   FirebaseJson sensorData;
//!   sensorData.set("temperature", temperature);
//!   sensorData.set("humidity", humidity);
//!   sensorData.set("light", mappedLuxGauge);
//!   sensorData.set("moisture", mappedMoistureLevel);
//!   sensorData.set("pumpActive", pumpActive ? 1 : 0);
//!
//!   if (Firebase.set(fbdo, logPath.c_str(), sensorData))
//!     Serial.println("Logged sensor data to Firebase.");
//!   else
//!     Serial.println(fbdo.errorReason());
//!
//!   delay(1000);
//! }
//!
//! /******************************************
//!   Supporting Functions
//! ******************************************/
//! void handleLCDControl(bool effectiveLCDSwitch) {
//!   if (effectiveLCDSwitch && !lcdActive) {
//!     lcdActive = true;
//!     lcdStartTime = millis();
//!     lcd.backlight();
//!     lcd.clear();
//!   }
//!   if (!effectiveLCDSwitch && lcdActive) {
//!     lcd.noBacklight();
//!     lcd.clear();
//!     lcdActive = false;
//!   }
//!   if (lcdActive && (millis() - lcdStartTime >= LCD_DURATION)) {
//!     lcd.noBacklight();
//!     lcd.clear();
//!     lcdActive = false;
//!   }
//! }
//!
//! void handlePumpControl(bool effectivePumpSwitch, int moisture) {
//!   if (effectivePumpSwitch || moisture < MOISTURE_THRESHOLD) {
//!     if (!pumpActive) {
//!       pumpActive = true;
//!       pumpStartTime = millis();
//!       digitalWrite(WATER_PUMP_PIN, HIGH);
//!       tone(BUZZER_PIN, 1000);
//!     } else if (millis() - pumpStartTime >= 2000) {
//!       noTone(BUZZER_PIN);
//!     }
//!   } else {
//!     if (pumpActive) {
//!       pumpActive = false;
//!       digitalWrite(WATER_PUMP_PIN, LOW);
//!       noTone(BUZZER_PIN);
//!     }
//!   }
//! }
//!
//! void printSensorValues(float temperature, float humidity, int lightVal, int moistureVal) {
//!   Serial.println("-----------------------------");
//!   Serial.print("Temperature: "); Serial.println(temperature);
//!   Serial.print("Humidity: "); Serial.println(humidity);
//!   Serial.print("Light: "); Serial.println(lightVal);
//!   Serial.print("Moisture: "); Serial.println(moistureVal);
//! }
//!
//! void updateLcd(float temperature, float humidity, int lightVal, int moistureVal) {
//!   lcd.setCursor(0, 0); lcd.print("Temp: "); lcd.print(temperature, 1); lcd.print("C ");
//!   lcd.setCursor(11, 0); lcd.print("Hum: "); lcd.print(humidity, 1); lcd.print("%");
//!   lcd.setCursor(0, 1); lcd.print("Light: "); lcd.print(lightVal);
//!   lcd.setCursor(0, 2); lcd.print("Moisture: "); lcd.print(moistureVal);
//!   lcd.setCursor(0, 3); lcd.print("Pump: "); lcd.print(pumpActive ? "ON " : "OFF");
//! }
//!
//! void handleLightAlert(int lightVal) {
//!   digitalWrite(LIGHT_ALERT_PIN, lightVal < LIGHT_THRESHOLD ? HIGH : LOW);
//! }
//!
//! void checkTemperatureAlert(float temperature) {
//!   if (temperature > TEMP_HIGH_THRESHOLD && !highNotified) {
//!     Blynk.virtualWrite(V8, "🌡️ High Temp Alert - Too hot!");
//!     highNotified = true; lowNotified = false;
//!   } else if (temperature < TEMP_LOW_THRESHOLD && !lowNotified) {
//!     Blynk.virtualWrite(V8, "🥶 Low Temp Alert - Too cold!");
//!     lowNotified = true; highNotified = false;
//!   } else if (temperature <= TEMP_HIGH_THRESHOLD && temperature >= TEMP_LOW_THRESHOLD) {
//!     if (highNotified || lowNotified) Blynk.virtualWrite(V8, "Temperature normalized");
//!     highNotified = lowNotified = false;
//!   }
//! }
//!
//! void checkHumidityNotification(float humidity) {
//!   if (humidity > 80.0 && !highHumidityNotified) {
//!     Blynk.virtualWrite(V10, "🌫️ High Humidity Alert - Air too humid!");
//!     highHumidityNotified = true; lowHumidityNotified = false;
//!   } else if (humidity < 30.0 && !lowHumidityNotified) {
//!     Blynk.virtualWrite(V10, "💧 Low Humidity Alert - Air too dry for plants!");
//!     lowHumidityNotified = true; highHumidityNotified = false;
//!   } else if (humidity >= 30.0 && humidity <= 80.0) {
//!     if (highHumidityNotified || lowHumidityNotified) Blynk.virtualWrite(V10, "Humidity normalised");
//!     highHumidityNotified = lowHumidityNotified = false;
//!   }
//! }
//!
//! void checkLightNotification(int lightVal) {
//!   if (lightVal < 100 && !lowLightNotified) {
//!     Blynk.virtualWrite(V11, "🌑 Low Light Alert - Too dark for plants!");
//!     lowLightNotified = true; highLightNotified = false;
//!   } else if (lightVal > 2200 && !highLightNotified) {
//!     Blynk.virtualWrite(V11, "☀️ High Light Alert - Too much sunlight!");
//!     highLightNotified = true; lowLightNotified = false;
//!   } else if (lightVal >= 100 && lightVal <= 2200) {
//!     if (lowLightNotified || highLightNotified) Blynk.virtualWrite(V11, "Light normalised");
//!     lowLightNotified = highLightNotified = false;
//!   }
//! }
//!
//! void checkMoistureNotification(int level) {
//!   if (level < 30 && !lowMoisture) {
//!     Blynk.virtualWrite(V12, "🌱 Dry Soil Alert - Water your plant!");
//!     lowMoisture = true; highMoisture = false;
//!   } else if (level > 80 && !highMoisture) {
//!     Blynk.virtualWrite(V12, "💦 Wet Soil Alert - Too much water!");
//!     highMoisture = true; lowMoisture = false;
//!   } else if (level >= 30 && level <= 80) {
//!     if (lowMoisture || highMoisture) Blynk.virtualWrite(V12, "Moisture normalised");
//!     lowMoisture = highMoisture = false;
//!   }
//! }

use esp_hal::{
    analog::adc::{Adc, AdcChannel, AdcPin, RegisterAccess},
    gpio::{Output, Input},
};

/// void handlePumpControl(bool effectivePumpSwitch, int moisture) {
///   if (effectivePumpSwitch || moisture < MOISTURE_THRESHOLD) {
///     if (!pumpActive) {
///       pumpActive = true;
///       pumpStartTime = millis();
///       digitalWrite(WATER_PUMP_PIN, HIGH);
///       tone(BUZZER_PIN, 1000);
///     } else if (millis() - pumpStartTime >= 2000) {
///       noTone(BUZZER_PIN);
///     }
///   } else {
///     if (pumpActive) {
///       pumpActive = false;
///       digitalWrite(WATER_PUMP_PIN, LOW);
///       noTone(BUZZER_PIN);
///     }
///   }
/// }
pub fn handle_pump_control(moisture: f32, user_switch: &mut Input, water_pump_pin: &mut Output) -> bool {
    if user_switch.is_low() || moisture < crate::MOISTURE_THRESHOLD {
        water_pump_pin.set_high();
        true
    } else {
        water_pump_pin.set_low();
        false
    }
}

/// void printSensorValues(float temperature, float humidity, int lightVal, int moistureVal) {
///   Serial.println("-----------------------------");
///   Serial.print("Temperature: "); Serial.println(temperature);
///   Serial.print("Humidity: "); Serial.println(humidity);
///   Serial.print("Light: "); Serial.println(lightVal);
///   Serial.print("Moisture: "); Serial.println(moistureVal);
/// }
pub fn print_sensor_values(temperature: f32, humidity: f32, light: f32, moisture: f32) {
    log::info!("-----------------------------");
    log::info!("Temperature: {temperature}");
    log::info!("Humidity: {humidity}");
    log::info!("Light: {light}");
    log::info!("Moisture: {moisture}");
}

pub fn read_light_lux<P, ADCI>(adc: &mut Adc<ADCI>, ldr_sensor_pin: &mut AdcPin<P, ADCI>) -> f32
where
    P: AdcChannel,
    ADCI: RegisterAccess,
{
    let raw_light_reading = match nb::block!(adc.read_oneshot(ldr_sensor_pin)) {
        Ok(r) => r,
        Err(err) => panic!("failed to read analog sensor: {:?}", err),
    };
    let raw_light_reading =
        (f32::from(raw_light_reading) + crate::LDR_ADC_RANGE_OFFSET).clamp(0.0, 4096.0);
    let voltage = raw_light_reading / 4096.0 * crate::LDR_VCC_VOLTAGE;
    let resistance = 2000.0 * voltage / (1.0 - voltage / crate::LDR_VCC_VOLTAGE);
    libm::powf(
        crate::LDR_RL10 * 1e3_f32 * libm::powf(10.0, crate::LDR_GAMMA) / resistance,
        1.0 / crate::LDR_GAMMA,
    )
}

// void handleLightAlert(int lightVal) {
//   digitalWrite(LIGHT_ALERT_PIN, lightVal < LIGHT_THRESHOLD ? HIGH : LOW);
// }
pub fn handle_light_alert(light_lux: f32, light_alert_pin: &mut Output) {
    light_alert_pin.set_level((light_lux < crate::LIGHT_ALERT_THRESHOLD).into());
}

// void checkTemperatureAlert(float temperature) {
//   if (temperature > TEMP_HIGH_THRESHOLD && !highNotified) {
//     Blynk.virtualWrite(V8, "🌡️ High Temp Alert - Too hot!");
//     highNotified = true; lowNotified = false;
//   } else if (temperature < TEMP_LOW_THRESHOLD && !lowNotified) {
//     Blynk.virtualWrite(V8, "🥶 Low Temp Alert - Too cold!");
//     lowNotified = true; highNotified = false;
//   } else if (temperature <= TEMP_HIGH_THRESHOLD && temperature >= TEMP_LOW_THRESHOLD) {
//     if (highNotified || lowNotified) Blynk.virtualWrite(V8, "Temperature normalized");
//     highNotified = lowNotified = false;
//   }
// }
//
// void checkHumidityNotification(float humidity) {
//   if (humidity > 80.0 && !highHumidityNotified) {
//     Blynk.virtualWrite(V10, "🌫️ High Humidity Alert - Air too humid!");
//     highHumidityNotified = true; lowHumidityNotified = false;
//   } else if (humidity < 30.0 && !lowHumidityNotified) {
//     Blynk.virtualWrite(V10, "💧 Low Humidity Alert - Air too dry for plants!");
//     lowHumidityNotified = true; highHumidityNotified = false;
//   } else if (humidity >= 30.0 && humidity <= 80.0) {
//     if (highHumidityNotified || lowHumidityNotified) Blynk.virtualWrite(V10, "Humidity normalised");
//     highHumidityNotified = lowHumidityNotified = false;
//   }
// }
//
// void checkLightNotification(int lightVal) {
//   if (lightVal < 100 && !lowLightNotified) {
//     Blynk.virtualWrite(V11, "🌑 Low Light Alert - Too dark for plants!");
//     lowLightNotified = true; highLightNotified = false;
//   } else if (lightVal > 2200 && !highLightNotified) {
//     Blynk.virtualWrite(V11, "☀️ High Light Alert - Too much sunlight!");
//     highLightNotified = true; lowLightNotified = false;
//   } else if (lightVal >= 100 && lightVal <= 2200) {
//     if (lowLightNotified || highLightNotified) Blynk.virtualWrite(V11, "Light normalised");
//     lowLightNotified = highLightNotified = false;
//   }
// }
//
// void checkMoistureNotification(int level) {
//   if (level < 30 && !lowMoisture) {
//     Blynk.virtualWrite(V12, "🌱 Dry Soil Alert - Water your plant!");
//     lowMoisture = true; highMoisture = false;
//   } else if (level > 80 && !highMoisture) {
//     Blynk.virtualWrite(V12, "💦 Wet Soil Alert - Too much water!");
//     highMoisture = true; lowMoisture = false;
//   } else if (level >= 30 && level <= 80) {
//     if (lowMoisture || highMoisture) Blynk.virtualWrite(V12, "Moisture normalised");
//     lowMoisture = highMoisture = false;
//   }
// }

#[derive(Default)]
pub struct NotificationChecker {
    light: f32,
    moisture: f32,
}

impl NotificationChecker {
    pub fn check_light(&mut self, value: f32) {
        if value < crate::LIGHT_LOW_THRESHOLD && value < self.light {
            log::warn!("🌑 Low Light Alert - Too dark for plants!");
        } else if value > crate::LIGHT_HIGH_THRESHOLD && value > self.light {
            log::warn!("☀️ High Light Alert - Too much sunlight!");
        } else if self
            .light
            .clamp(crate::LIGHT_LOW_THRESHOLD, crate::LIGHT_HIGH_THRESHOLD)
            != self.light
        {
            log::warn!("Light normalised")
        }
        self.light = value;
    }

    pub fn check_moisture(&mut self, value: f32) {
        if value < crate::MOISTURE_LOW_THRESHOLD && value < self.moisture {
            log::warn!("🌱 Dry Soil Alert - Water your plant!");
        } else if value > crate::MOISTURE_HIGH_THRESHOLD && value > self.moisture {
            log::warn!("💦 Wet Soil Alert - Too much water!");
        } else if self.moisture.clamp(
            crate::MOISTURE_LOW_THRESHOLD,
            crate::MOISTURE_HIGH_THRESHOLD,
        ) != self.moisture
        {
            log::info!("Moisture normalised");
        }
        self.moisture = value;
    }
}
