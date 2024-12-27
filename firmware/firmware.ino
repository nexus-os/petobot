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

const int LEFT_SENSOR = A0;
const int RIGHT_SENSOR = A1;

const int TURN_DELAY_MS = 200;

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

MyServerCallbacks cbs;

void setup() {
  Serial.begin(115200);

  pinMode(leftPin, OUTPUT);
  pinMode(rightPin, OUTPUT);
  pinMode(ledPin, OUTPUT);
  pinMode(LEFT_SENSOR, INPUT_PULLUP);
  pinMode(RIGHT_SENSOR, INPUT_PULLUP);

  analogWrite(leftPin, 0);
  analogWrite(rightPin, 0);
  analogWrite(ledPin, 0);

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

void setMotors(bool left, bool right) {
  analogWrite(leftPin, left ? 255 : 0);
  analogWrite(rightPin, right ? 255 : 0);
  Serial.printf("Motors L:%d R:%d\n", left, right);
}

void loop() {
  Serial.println("Peto™");
  int leftSensor = digitalRead(LEFT_SENSOR);
  int rightSensor = digitalRead(RIGHT_SENSOR);

  bool leftDetected = leftSensor == LOW;
  bool rightDetected = rightSensor == LOW;

  if (!leftDetected && !rightDetected) {
    // No line detected - go forward
    setMotors(true, true);
  } else if (leftDetected && !rightDetected) {
    // Line on left - turn right
    setMotors(true, false);
    delay(TURN_DELAY_MS);
  } else if (!leftDetected && rightDetected) {
    // Line on right - turn left
    setMotors(false, true);
    delay(TURN_DELAY_MS);
  } else {
    // Both sensors on line - stop
    setMotors(false, false);
  }

  delay(50);  // debounce
}
