build:
	arduino-cli compile --fqbn esp32:esp32:esp32c3 firmware

upload:
	arduino-cli upload -p /dev/ttyACM0 --fqbn esp32:esp32:esp32c3 firmware

server:
	cargo run

run: build upload server