# AHT20 Sensor Module Driver

## Descripion

A driver crate for the AHT20 Temperture and Humidity modules/sensors.

The library is written in rust using TDD and making heavy use of the 
`embedded-hal-mock` crate to mock away the dependency on i2c hardware.

This crate is made by referencing the C code and multipl versions of the 
datashees, as there is a v1.0 and v1.1 datasheet availble along with various
other datasheets with varying formatting and info.

There seems to be some translation mistakes in the datasheets from what I can
tell so I've made some assumptions where needed.


## Running tests

```sh
cargo test
```


## Usage

## Overview

