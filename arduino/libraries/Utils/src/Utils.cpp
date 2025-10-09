#include "./Utils.h"

#include <WiFi.h>

void format_encryption_type(int enc_type) {
  switch (enc_type) {
    case WIFI_AUTH_OPEN:
        Serial.print("open");
        break;
    case WIFI_AUTH_WEP:
        Serial.print("WEP");
        break;
    case WIFI_AUTH_WPA_PSK:
        Serial.print("WPA");
        break;
    case WIFI_AUTH_WPA2_PSK:
        Serial.print("WPA2");
        break;
    case WIFI_AUTH_WPA_WPA2_PSK:
        Serial.print("WPA+WPA2");
        break;
    case WIFI_AUTH_WPA2_ENTERPRISE:
        Serial.print("WPA2-EAP");
        break;
    case WIFI_AUTH_WPA3_PSK:
        Serial.print("WPA3");
        break;
    case WIFI_AUTH_WPA2_WPA3_PSK:
        Serial.print("WPA2+WPA3");
        break;
    case WIFI_AUTH_WAPI_PSK:
        Serial.print("WAPI");
        break;
    default:
        Serial.print("unknown");
  }
  Serial.println();
}

const char* format_wifi_status(int status) {
    switch (status) {
        case WL_NO_SHIELD:
            return "no-shield";
        case WL_IDLE_STATUS:
            return "idle";
        case WL_NO_SSID_AVAIL:
            return "no-ssid-available";
        case WL_SCAN_COMPLETED:
            return "scan-completed";
        case WL_CONNECTED:
            return "connected";
        case WL_CONNECT_FAILED:
            return "connect-failed";
        case WL_CONNECTION_LOST:
            return "connection-lost";
        case WL_DISCONNECTED:
            return "disconnected";
        default:
            return "unknown";
    }
}

void scan_networks() {
    // scan for nearby networks:
    Serial.println("** Scan Networks **");
    int numSsid = WiFi.scanNetworks();
    if (numSsid == -1) {
      Serial.println("Couldn't get a wifi connection");
      while (true);
    }

    // print the list of networks seen:
    Serial.print("number of available networks:");
    Serial.println(numSsid);

    // print the network number and name for each network found:
    for (int network_id = 0; network_id < numSsid; network_id++) {
      Serial.print(network_id);
      Serial.print(") ");
      Serial.print(WiFi.SSID(network_id));
      Serial.print("\tSignal: ");
      Serial.print(WiFi.RSSI(network_id));
      Serial.print(" dBm");
      Serial.print("\tEncryption: ");
      format_encryption_type(WiFi.encryptionType(network_id));
    }
}

void utils::connect_wifi(const char* ssid, const char* password, bool perform_scan) {
  // check for the presence of the shield:
  if (WiFi.status() == WL_NO_SHIELD) {
    Serial.println("WiFi shield not present");
    // don't continue:
    while (true);
  }

  if (perform_scan) {
    scan_networks();
  }

  // attempt to connect to Wifi network:
  int wifi_status = WiFi.begin(ssid, password);
  while (wifi_status != WL_CONNECTED) {
    int new_wifi_status = WiFi.status();
    if (new_wifi_status != wifi_status) {
        Serial.print("Wifi status: ");
        Serial.println(format_wifi_status(new_wifi_status));
    }
    wifi_status = new_wifi_status;

    delay(2000);
  }

  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());
}
