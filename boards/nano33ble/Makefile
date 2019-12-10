# Makefile for building the tock kernel for the Arduino Nano 33 BLE board.

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabi
PLATFORM=nano33ble

include ../Makefile.common

# Upload programs over uart with tockloader
ifdef PORT
  FLAGS += --port $(PORT)
endif

# Upload the kernel over serial/bootloader
.PHONY: program
program: target/$(TARGET)/release/$(PLATFORM).bin
	bossac $(FLAGS) -U -i -e -w $< -R
# 	bossac --port=/dev/cu.usbmodem14201 -U -i -e -w target/thumbv7em-none-eabi/release/nrf52840dk.bin -R -d
