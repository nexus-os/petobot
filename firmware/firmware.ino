#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEServer.h>
#include <BLE2902.h>

// BLE UUIDs
#define SERVICE_UUID           "08daa714-ccf1-42a8-8a88-535652d04bac" // UUID for the BLE service
#define CHARACTERISTIC_UUID_LEFT  "1EF71EF7-1EF7-1EF7-1EF7-1EF71EF71EF7" // UUID for X
#define CHARACTERISTIC_UUID_RIGHT  "1E551EF7-1E55-1E55-1E55-1E551E551EF7" // UUID for Y

constexpr int LEFT_PIN = 5;
constexpr int RIGHT_PIN = 3;
constexpr int LED_PIN = 8;

constexpr int LEFT_SENSOR = 0;
constexpr int RIGHT_SENSOR = 1;

constexpr int TURN_DELAY_MS = 200;

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

void loop() {
  Serial.println("Peto!™");
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

// vim: set et ts=2 sw=2:
