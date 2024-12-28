#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEServer.h>
#include <BLE2902.h>

// BLE UUIDs
#define SERVICE_UUID              "08daa714-ccf1-42a8-8a88-535652d04bac" // UUID for the BLE service
#define CHARACTERISTIC_UUID_LEFT  "1EF71EF7-1EF7-1EF7-1EF7-1EF71EF71EF7" // UUID for X
#define CHARACTERISTIC_UUID_RIGHT "1E551EF7-1E55-1E55-1E55-1E551E551EF7" // UUID for Y
#define CHARACTERISTIC_UUID_GO    "FA57FA57-FA57-FA57-FA57-FA57FA57FA57" // UUID to start
#define CHARACTERISTIC_UUID_STOP  "1E55FA57-1E55-FA57-1E55-FA571E55FA57" // UUID to unstart

static constexpr int LEFT_PIN = 5;
static constexpr int RIGHT_PIN = 3;
static constexpr int LED_PIN = 8;

static constexpr int LEFT_SENSOR = 2;
static constexpr int RIGHT_SENSOR = 10;

static constexpr int TURN_DELAY_MS = 200;

static BLECharacteristic *leftCharacteristic;
static BLECharacteristic *rightCharacteristic;
static BLECharacteristic *goCharacteristic;
static BLECharacteristic *stopCharacteristic;

static bool moving = false;

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
        digitalWrite(LEFT_PIN, value);
      } else if (c == rightCharacteristic) {
        digitalWrite(RIGHT_PIN, value);
      } else if (c == goCharacteristic) {
        moving = true;
      } else if (c == stopCharacteristic) {
        moving = false;
      }
      Serial.print(buf);
      Serial.print("\n");
    }
};
MyServerCallbacks cbs;

void setup() {
  Serial.begin(115200);

  pinMode(LEFT_PIN, OUTPUT);
  pinMode(RIGHT_PIN, OUTPUT);
  pinMode(LED_PIN, OUTPUT);
  pinMode(LEFT_SENSOR, INPUT_PULLUP);
  pinMode(RIGHT_SENSOR, INPUT_PULLUP);

  digitalWrite(LEFT_PIN, LOW);
  digitalWrite(RIGHT_PIN, LOW);
  digitalWrite(LED_PIN, LOW);

  Serial.println("Peto™");
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
  goCharacteristic = pService->createCharacteristic(
                       CHARACTERISTIC_UUID_GO,
                       BLECharacteristic::PROPERTY_WRITE | BLECharacteristic::PROPERTY_NOTIFY
                     );
  goCharacteristic->setCallbacks(&cbs);
  stopCharacteristic = pService->createCharacteristic(
                         CHARACTERISTIC_UUID_STOP,
                         BLECharacteristic::PROPERTY_WRITE | BLECharacteristic::PROPERTY_NOTIFY
                       );
  stopCharacteristic->setCallbacks(&cbs);

  pService->start();
  BLEAdvertising *pAdvertising = pServer->getAdvertising();
  pAdvertising->addServiceUUID(SERVICE_UUID);
  pAdvertising->start();

  spettacolino();
  Serial.println("Peto2™");
}

void setMotors(bool left, bool right) {
  digitalWrite(LEFT_PIN, left ? HIGH : LOW);
  digitalWrite(RIGHT_PIN, right ? HIGH : LOW);
  Serial.printf("Motors L:%d R:%d\n", left, right);
}

void spettacolino() {
  for (int i = 0; i < 5; ++i) {
    digitalWrite(LED_PIN, HIGH);
    delay(250);
    digitalWrite(LED_PIN, LOW);
    delay(250);
  }
}

void move() {
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
}

void loop() {
  Serial.println("Peto!™");
  if (moving) {
    move();
  }
  delay(50);  // debounce
}

// vim: set et ts=2 sw=2:
