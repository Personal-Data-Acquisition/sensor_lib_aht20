# AHT20 Sensor Module Driver

## Description

A driver crate for the AHT20 Temperature and Humidity modules/sensors.

The library is written in rust using TDD and making heavy use of the 
`embedded-hal-mock` crate to mock away the dependency on i2c hardware.

This crate is made by referencing the C code and multiple versions of the 
data sheets, as there is a v1.0 and v1.1 data sheet available along with various
other data sheets with varying formatting and info.

There seems to be some translation mistakes in the data sheets from what I can
tell so I've made some assumptions where needed.

**CRC stuff**
One version of the datasheet seems to describe a 8bit CRC check being used
for the readings that come  from the sensor.

The polynomial described is `0x31` and an initial value of `0xFF`. This sounds
like the `CRC-8-MAXIM`. So that is what this driver uses.


## Running tests

This assumes you are not in the midst of using this repo as a git-submodule.
If you are then you may need to specify the target for your development 
machine using the `rustup target list` command to find the correct target.

```sh
cargo test
```


## Usage



## Overview


## TODO:

- [x] Upgrade to newest embedded-hal/mock
- [x] Setup CRC8 for i2c
- [x] Build out the methods to read sensor data.
- [x] Update to poll for finished CRC calculations from sensor.
- [ ] Add documentation for recomendations of i2c reliability.
- [ ] Impliment generic standard for sensor info/formatting.
- [ ] Reduce the size of the lib.rs file.
- [ ] Look into making the driver non-blocking.





