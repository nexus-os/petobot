#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEServer.h>
#include <BLE2902.h>

// BLE UUIDs
#define SERVICE_UUID           "08daa714-ccf1-42a8-8a88-535652d04bac" // UUID for the BLE service
#define CHARACTERISTIC_UUID_LEFT  "1EF71EF7-1EF7-1EF7-1EF7-1EF71EF71EF7" // UUID for X
#define CHARACTERISTIC_UUID_RIGHT  "1E551EF7-1E55-1E55-1E55-1E551E551EF7" // UUID for Y


int leftPin = 5;
int rightPin = 3;
int ledPin = 8;

BLECharacteristic *leftCharacteristic;
BLECharacteristic *rightCharacteristic;

class MyServerCallbacks: public BLEServerCallbacks, public BLECharacteristicCallbacks {
    void onConnect(BLEServer* pServer) {
      Serial.println("Client Connected");
    };

    void onDisconnect(BLEServer* pServer) {
      Serial.println("Client Disconnected");
      BLEDevice::startAdvertising();
    }
    void onWrite(BLECharacteristic *c) {
      uint8_t value = c->getData()[0];
      char buf[4];
      snprintf(buf, sizeof(buf), "%u", value);
      if (c == leftCharacteristic) {
        analogWrite(leftPin, value);
      } else if (c == rightCharacteristic) {
        analogWrite(rightPin, value);
      }
      Serial.print(buf);
      Serial.print("\n");
    }
};

bool ledOn;
MyServerCallbacks cbs;

void toggleLed() {
  ledOn = !ledOn;
  if (ledOn) {
    digitalWrite(ledPin, LOW);
  } else {
    digitalWrite(ledPin, HIGH);
  }
}

void setup() {
  Serial.begin(115200);

  pinMode(leftPin, OUTPUT);
  pinMode(rightPin, OUTPUT);
  pinMode(ledPin, OUTPUT);

  analogWrite(leftPin, 0);
  analogWrite(rightPin, 0);
  digitalWrite(ledPin, LOW);
  ledOn = true;
  for (int i = 0; i < 10; ++i) {
    toggleLed();
    delay(200);
    log_e("BANANA");
  }


  BLEDevice::init("Petobot™");
    BLEServer *pServer = BLEDevice::createServer();

    pServer->setCallbacks(&cbs);

    BLEService *pService = pServer->createService(SERVICE_UUID);
    leftCharacteristic = pService->createCharacteristic(
                         CHARACTERISTIC_UUID_LEFT,
                         BLECharacteristic::PROPERTY_WRITE | BLECharacteristic::PROPERTY_NOTIFY
                       );
    leftCharacteristic->setCallbacks(&cbs);
    rightCharacteristic = pService->createCharacteristic(
                          CHARACTERISTIC_UUID_RIGHT,
                          BLECharacteristic::PROPERTY_WRITE | BLECharacteristic::PROPERTY_NOTIFY
                        );
    rightCharacteristic->setCallbacks(&cbs);

    pService->start();
    BLEAdvertising *pAdvertising = pServer->getAdvertising();
    pAdvertising->addServiceUUID(SERVICE_UUID);
    pAdvertising->start();
}

void loop() {
  Serial.println("Peto™");

  delay(1000);
  toggleLed();
}
