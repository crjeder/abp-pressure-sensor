# abp-pressure-sensor

[![Crate](https://img.shields.io/crates/v/abp-pressure-sensor?style=plastic)](https://crates.io/crates/abp-pressure-sensor)
![License](https://img.shields.io/crates/l/abp-pressure-sensor?style=plastic)
![GitHub branch checks state](https://img.shields.io/github/checks-status/crjeder/abp-pressure-sensor/release?style=plastic)
<!--![Docs](https://img.shields.io/docsrs/abp-pressure-sensor?style=plastic)-->
<!--![LOC](https://img.shields.io/tokei/lines/github/crjeder/abp-pressure-sensor?style=plastic)-->
![Maintained](https://img.shields.io/maintenance/yes/2022?style=plastic)
[![dependency status](https://deps.rs/repo/github/crjeder/abp-pressure-sensor/status.svg)](https://deps.rs/repo/github/crjeder/abp-pressure-sensor)
![GitHub Repo stars](https://img.shields.io/github/stars/crjeder/abp-pressure-sensor?style=plastic)
![Crates.io](https://img.shields.io/crates/d/abp-pressure-sensor?style=plastic)
<!-- [![crev reviews](https://web.crev.dev/rust-reviews/badge/crev_count/abp-pressure-sensor_bb.png)](https://web.crev.dev/rust-reviews/crate/abp-pressure-sensor/)-->

This is a platform agnostic driver to interface with Honeywells APB line of pressure sensors (https://sps.honeywell.com/gb/en/products/advanced-sensing-technologies/healthcare-sensing/board-mount-pressure-sensors/basic-abp-series)
This `no_std` driver is built on [`embedded-hal`][2] **1.0** traits.

## Usage

Add to `Cargo.toml`:

```toml
abp-pressure-sensor = "0.2"
```

Instantiate with your HAL's I2C and delay types, plus the full Honeywell part number string:

```rust
use abp_pressure_sensor::Abp;

// e.g. using stm32f4xx-hal ≥ 0.21
let mut sensor = Abp::new(i2c, delay, "ABPDNNN030PG2A3");

// Read pressure in Pascals
let pressure_pa = sensor.read()?;

// Read pressure (Pa) and temperature (°C) together (sensors with thermometer only)
let (pressure_pa, temp_c) = sensor.pressure_and_temperature()?;
```

## Migration from 0.1.x

| Change | 0.1.x | 0.2.x |
|--------|-------|-------|
| `embedded-hal` version | 0.2.7 | **1.0.0** |
| Error type | `ApbError<E>` | `AbpError<E>` (typo fixed) |
| Stale-data error | `nb::Error::WouldBlock` | `AbpError::DataNotReady` |
| `read()` return | `nb::Result<f32, …>` | `Result<f32, AbpError<…>>` |
| `pressure_and_temperature()` return | `Result<f32, E>` | `Result<(f32, f32), AbpError<…>>` |

Update your HAL board-support crate to a version that implements `embedded-hal 1.0`
(e.g. `rppal ≥ 0.18`, `stm32f4xx-hal ≥ 0.21`, `rp2040-hal ≥ 0.10`).

## Example

### Wiring

### Code

## What works
(tested on Raspberry Pi)

  -

## TODO

  - [ ] Test on more platforms
  -


## Feedback
All kind of feedback is welcome. If you have questions or problems, please post them on the issue tracker
This is literally the first code I ever wrote in rust. I am still learning. So please be patient, it might take me some time to fix a bug. I may have to break my knowledge sound-barrier.
If you have tested on another platform I'd like to hear about that, too!

# References

  - [datasheet][1]

[1]: https://prod-edam.honeywell.com/content/dam/honeywell-edam/sps/siot/en-gb/products/sensors/pressure-sensors/board-mount-pressure-sensors/basic-abp-series/documents/sps-siot-basic-board-mount-pressure-abp-series-datasheet-32305128-ciid-155789.pdf

  - [embedded-hal][2]

[2]: https://github.com/rust-embedded/embedded-hal

## License

Licensed under either of

  - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
  - MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
