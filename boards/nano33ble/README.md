Arduino Nano 33 BLE
===================

The [Arduino Nano 33 BLE](https://store.arduino.cc/usa/nano-33-ble) and [Arduino
Nano 33 BLE Sense](https://store.arduino.cc/usa/nano-33-ble-sense) are compact
boards based on the Nordic nRF52840 SoC. The "Sense" version includes the
following sensors:

- 9 axis inertial sensor
- humidity and temperature sensor
- barometric sensor
- microphone
- gesture, proximity, light color and light intensity sensor


## Getting Started

First, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

You will need the bossac bootloader tool:

```shell
git clone https://github.com/arduino/BOSSA
cd BOSSA
make bossac
```

## Programming the kernel
Once you have all software installed, you should be able to simply run
`make program` in this directory to install a fresh kernel.

## Programming user-level applications
You can program an application via JTAG using `tockloader`:

    ```bash
    $ cd libtock-c/examples/<app>
    $ make
    $ tockloader install --jlink --board nrf52dk
    ```

## Debugging

See the [nrf52dk README](../nrf52dk/README.md) for information about debugging
the nRF52840dk.
