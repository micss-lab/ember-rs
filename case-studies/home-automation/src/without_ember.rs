//! #include <LiquidCrystal_I2C.h>
//! #include <WiFi.h>
//! #include <WiFiClient.h>
//! #include "DHTesp.h"
//!
//! #define BLYNK_TEMPLATE_ID "TMPL3vRehCOGh"
//! #define BLYNK_TEMPLATE_NAME "smart home"
//! #define BLYNK_AUTH_TOKEN "WyhqRB2NDLX6RxfGA6xTY0tECI760jAW"
//! #define BLYNK_PRINT Serial
//! #include <BlynkSimpleEsp32.h>
//!
//! #define LED_PIN 33
//! #define INPUT_PIN 27
//! #define FAN_PIN 5
//! #define LED_2_PIN 2
//! #define LCD_DATA_PIN 21
//! #define IR_PIN 23
//! #define PIR_PIN 27
//! #define TEMP_SENSOR_PIN 15
//!
//! #define WIFI_SSID "Wokwi-GUEST"
//! #define WIFI_PASSWORD ""
//!
//! byte TEMP_SYMBOL[4][8] = {
//!   { B00000, B00001, B00010, B00100, B00100, B00100, B00100, B00111 },
//!   { B00111, B00111, B00111, B01111, B11111, B11111, B01111, B00011 },
//!   { B00000, B10000, B01011, B00100, B00111, B00100, B00111, B11100 },
//!   { B11111, B11100, B11100, B11110, B11111, B11111, B11110, B11000 },
//! };
//!
//! byte HUMIDITY_SYMBOL[4][8] = {
//!   { B00000, B00001, B00011, B00011, B00111, B01111, B01111, B11111 },
//!   { B11111, B11111, B11111, B01111, B00011, B00000, B00000, B00000 },
//!   { B00000, B10000, B11000, B11000, B11100, B11110, B11110, B11111 },
//!   { B11111, B11111, B11111, B11110, B11100, B00000, B00000, B00000 },
//! };
//!
//! byte HOUSE_SYMBOL[4][8] = {
//!   { B00000, B00001, B00011, B00011, B00111, B01111, B01111, B11111 },
//!   { B11111, B11111, B11100, B11100, B11100, B11100, B11100, B11100 },
//!   { B00000, B10010, B11010, B11010, B11110, B11110, B11110, B11111 },
//!   { B11111, B11111, B11111, B10001, B10001, B10001, B11111, B11111 },
//! };
//!
//! byte LOCK_SYMBOL[8] = { B01110, B10001, B10001, B11111, B11011, B11011, B11111, B00000 };
//!
//! byte DEGREE_SYMBOL[8] = { B00011, B00011, B00000, B00000, B00000, B00000, B00000, B00000 };
//!
//! LiquidCrystal_I2C lcd(0x27, 20, 4);
//! BlynkTimer timer;
//!
//! int ge;
//! int k;
//! float tmp = 0, hum = 0;
//!
//! bool led2State = false;
//! bool pirState = false;
//! bool fanState = false;
//!
//! DHTesp temps;
//!
//! BLYNK_WRITE(V0){
//!  fanState = param.asInt();
//!  digitalWrite(FAN_PIN, fanState);
//! }
//!
//! BLYNK_WRITE(V1) {
//!   pirState = param.asInt();
//!   digitalWrite(LED_PIN, pirState);
//!   k = !pirState;
//!   ge = pirState;
//! }
//!
//! BLYNK_WRITE(V2){
//!  led2State = param.asInt();
//!  digitalWrite(LED_2_PIN, led2State);
//! }
//!
//! void setup() {
//!   pinMode(FAN_PIN, OUTPUT);
//!   pinMode(IR_PIN, INPUT);
//!   pinMode(LED_2_PIN, OUTPUT);
//!   pinMode(LED_PIN, OUTPUT);      
//!   pinMode(INPUT_PIN, INPUT_PULLUP);
//!
//!   Serial.begin(115200);
//!   Blynk.begin(BLYNK_AUTH_TOKEN, WIFI_SSID, WIFI_PASSWORD);
//!   temps.setup(TEMP_SENSOR_PIN, DHTesp::DHT22);
//!
//!   lcd.init();
//!   lcd.backlight();
//!
//!   home_screen();
//!
//!   digitalWrite(FAN_PIN, LOW);
//!   digitalWrite(LCD_DATA_PIN, LOW);
//!
//!   delay(3000);
//!   
//!   Blynk.virtualWrite(V7, pirState);
//!   timer.setInterval(1000, [] {
//!     Blynk.virtualWrite(V3, tmp);
//!     Blynk.virtualWrite(V4, hum);
//!   });
//! }
//!
//! void loop() {
//!   Blynk.run();
//!   timer.run();
//!
//!   if(digitalRead(IR_PIN)) {  
//!     digitalWrite(LED_2_PIN, led2State);  
//!   } else {
//!     digitalWrite(LED_2_PIN, LOW);  
//!   }
//!
//!   TempAndHumidity  x = temps.getTempAndHumidity();
//!   tmp = x.temperature ;
//!   hum = x.humidity ;
//!
//!   if (digitalRead(INPUT_PIN)) {
//!     if (k == 1) {
//!       digitalWrite(LED_PIN, LOW);  
//!       k = 0 ;
//!       ge = 0;
//!     } else if (k == 0) {
//!       digitalWrite(LED_PIN, HIGH);
//!       k = 1;
//!       ge = 1;
//!     }
//!   }
//!
//!   
//!   switchboard_screen();
//!   delay(1500);
//!   temperature_screen();
//!   delay(750);
//!   humidity_screen();
//!   delay(750);
//! }
//!
//! void home_screen() {
//!   lcd.clear();
//!   reset_symbols();
//!
//!   draw_symbol(HOUSE_SYMBOL, 1, 2);
//!   draw_symbol(HOUSE_SYMBOL, 17, 2);
//!   draw_symbol(LOCK_SYMBOL, 19, 0);
//!
//!   lcd.setCursor(9,0);
//!   lcd.print("connected-");
//!   lcd.setCursor(2,1);
//!   lcd.print("HOME AUTOMATION");
//!   lcd.setCursor(6,2);
//!   lcd.print("USING IOT");
//! }
//!
//! void switchboard_screen() {
//!   lcd.clear();
//!   reset_symbols();
//!
//!   draw_symbol(LOCK_SYMBOL, 19, 0);
//!
//!   if (fanState){
//!     lcd.setCursor(0, 1);
//!     lcd.print("SW_1= ");
//!     lcd.print("ON ");
//!   } else {
//!     lcd.setCursor(0, 1);
//!     lcd.print("SW_1= ");
//!     lcd.print("OFF");
//!   }     
//!  
//!   if (led2State) {
//!     lcd.setCursor(0, 3);
//!     lcd.print("sw_2= ");
//!     lcd.print("ON ");
//!   } else {
//!     lcd.setCursor(0, 3);
//!     lcd.print("sw_2= ");
//!     lcd.print("OFF");
//!   }     
//!     
//!   if (ge == 1){
//!     lcd.setCursor(10, 3);
//!     lcd.print("lock= ");
//!     lcd.print("open ");
//!   } else {
//!     lcd.setCursor(11, 3);
//!     lcd.print("lock= ");
//!     lcd.print("cls");
//!   }
//! }
//!
//! void temperature_screen() {
//!   lcd.clear();
//!   reset_symbols();
//!
//!   draw_symbol(LOCK_SYMBOL, 19, 0);
//!   draw_symbol(TEMP_SYMBOL, 1, 1);
//!
//!   lcd.setCursor(4,1);
//!   lcd.print("Temperature :");
//!   lcd.setCursor(7,2);
//!   lcd.print(tmp);
//!   draw_symbol(DEGREE_SYMBOL, 11, 2);
//!   lcd.setCursor(12,2);
//!   lcd.print("C");
//! }
//!
//! void humidity_screen() {
//!   lcd.clear();
//!   reset_symbols();
//!
//!   draw_symbol(LOCK_SYMBOL, 19, 0);
//!   draw_symbol(HUMIDITY_SYMBOL, 3, 1);
//!
//!   lcd.setCursor(6,1);
//!   lcd.print("Humidity :");
//!   lcd.setCursor(7,2);
//!   lcd.print(hum);
//!   lcd.setCursor(12,2);
//!   lcd.print("%");
//! }
//!
//! unsigned int count = 0;
//! void draw_symbol(byte symbol[8], unsigned int x, unsigned int y) {
//!   lcd.createChar(count, symbol);
//!   lcd.setCursor(x, y);
//!   lcd.write(count++);
//! }
//!
//! void draw_symbol(byte symbol[4][8], unsigned int x, unsigned int y) {
//!   unsigned int start = count;
//!   lcd.createChar(count++, symbol[0]);
//!   lcd.createChar(count++, symbol[1]);
//!   lcd.createChar(count++, symbol[2]);
//!   lcd.createChar(count++, symbol[3]);
//!
//!   lcd.setCursor(x, y);
//!   lcd.write(start++);
//!   lcd.setCursor(x, y + 1);
//!   lcd.write(start++);
//!   lcd.setCursor(x + 1, y);
//!   lcd.write(start++);
//!   lcd.setCursor(x + 1, y + 1);
//!   lcd.write(start++);
//! }
//!
//! void reset_symbols() {
//!   count = 0;
//! }
