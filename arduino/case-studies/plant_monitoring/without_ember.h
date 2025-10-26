#ifndef USE_EMBER

/******************************************
  State Flags & Control
******************************************/
bool pumpActive = false;
unsigned long pumpStartTime = 0;
bool lcdActive = false;
unsigned long lcdStartTime = 0;
const unsigned long LCD_DURATION = 25000;

bool highNotified = false;
bool lowNotified = false;
bool highHumidityNotified = false;
bool lowHumidityNotified = false;
bool lowLightNotified = false;
bool highLightNotified = false;
bool virtualLCD = false;
bool virtualPump = false;
bool lowMoisture = false;
bool highMoisture = false;

/******************************************
  Supporting Functions
******************************************/
void handlePumpControl(bool effectivePumpSwitch, int moisture) {
  if (effectivePumpSwitch || moisture < MOISTURE_THRESHOLD) {
    if (!pumpActive) {
      pumpActive = true;
      pumpStartTime = millis();
      digitalWrite(WATER_PUMP_PIN, HIGH);
      tone(BUZZER_PIN, 1000);
    } else if (millis() - pumpStartTime >= 2000) {
      noTone(BUZZER_PIN);
    }
  } else {
    if (pumpActive) {
      pumpActive = false;
      digitalWrite(WATER_PUMP_PIN, LOW);
      noTone(BUZZER_PIN);
    }
  }
}

void printSensorValues(float temperature, float humidity, int lightVal, int moistureVal) {
  Serial.println("-----------------------------");
  Serial.print("Temperature: "); Serial.println(temperature);
  Serial.print("Humidity: "); Serial.println(humidity);
  Serial.print("Light: "); Serial.println(lightVal);
  Serial.print("Moisture: "); Serial.println(moistureVal);
}

void handleLightAlert(int lightVal) {
  digitalWrite(LIGHT_ALERT_PIN, lightVal < LIGHT_ALERT_THRESHOLD ? HIGH : LOW);
}

void checkTemperatureAlert(float temperature) {
  if (temperature > TEMP_HIGH_THRESHOLD && !highNotified) {
    Blynk.virtualWrite(V8, "🌡️ High Temp Alert - Too hot!");
    highNotified = true; lowNotified = false;
  } else if (temperature < TEMP_LOW_THRESHOLD && !lowNotified) {
    Blynk.virtualWrite(V8, "🥶 Low Temp Alert - Too cold!");
    lowNotified = true; highNotified = false;
  } else if (temperature <= TEMP_HIGH_THRESHOLD && temperature >= TEMP_LOW_THRESHOLD) {
    if (highNotified || lowNotified) Blynk.virtualWrite(V8, "Temperature normalized");
    highNotified = lowNotified = false;
  }
}

void checkHumidityNotification(float humidity) {
  if (humidity > 80.0 && !highHumidityNotified) {
    Blynk.virtualWrite(V10, "🌫️ High Humidity Alert - Air too humid!");
    highHumidityNotified = true; lowHumidityNotified = false;
  } else if (humidity < 30.0 && !lowHumidityNotified) {
    Blynk.virtualWrite(V10, "💧 Low Humidity Alert - Air too dry for plants!");
    lowHumidityNotified = true; highHumidityNotified = false;
  } else if (humidity >= 30.0 && humidity <= 80.0) {
    if (highHumidityNotified || lowHumidityNotified) Blynk.virtualWrite(V10, "Humidity normalised");
    highHumidityNotified = lowHumidityNotified = false;
  }
}

void checkLightNotification(int lightVal) {
  if (lightVal < 100 && !lowLightNotified) {
    Blynk.virtualWrite(V11, "🌑 Low Light Alert - Too dark for plants!");
    lowLightNotified = true; highLightNotified = false;
  } else if (lightVal > 2200 && !highLightNotified) {
    Blynk.virtualWrite(V11, "☀️ High Light Alert - Too much sunlight!");
    highLightNotified = true; lowLightNotified = false;
  } else if (lightVal >= 100 && lightVal <= 2200) {
    if (lowLightNotified || highLightNotified) Blynk.virtualWrite(V11, "Light normalised");
    lowLightNotified = highLightNotified = false;
  }
}

void checkMoistureNotification(int level) {
  if (level < 30 && !lowMoisture) {
    Blynk.virtualWrite(V12, "🌱 Dry Soil Alert - Water your plant!");
    lowMoisture = true; highMoisture = false;
  } else if (level > 80 && !highMoisture) {
    Blynk.virtualWrite(V12, "💦 Wet Soil Alert - Too much water!");
    highMoisture = true; lowMoisture = false;
  } else if (level >= 30 && level <= 80) {
    if (lowMoisture || highMoisture) Blynk.virtualWrite(V12, "Moisture normalised");
    lowMoisture = highMoisture = false;
  }
}

#endif // USE_EMBER