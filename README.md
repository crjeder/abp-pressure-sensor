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
abp-pressure-sensor = "0.3"
```

Parse the Honeywell part number to get an `AbpConfig`, then construct the driver:

```rust
use abp_pressure_sensor::{Abp, AbpConfig, TemperatureResolution};

// Parse config from the full part-number string (returns Result — no panics)
let config = AbpConfig::from_part_number("ABPDNNN030PG2A3").unwrap();
let mut sensor = Abp::new(i2c, config);

// Fast 2-byte read — pressure only (Pa)
let pressure_pa = sensor.read()?;

// Convert to native unit for display:  "30.0 psi"
let native = pressure_pa / sensor.config.unit.to_pa_factor();

// 4-byte read — pressure + 11-bit temperature (sensors with thermometer only)
let (pressure_pa, temp_opt) = sensor.read_with_temperature(TemperatureResolution::Full)?;
if let Some(temp_c) = temp_opt {
    // sensor.config.has_thermometer is true
}

// 3-byte read — pressure + 8-bit temperature (~0.8 °C, faster / lower power)
let (pressure_pa, temp_opt) = sensor.read_with_temperature(TemperatureResolution::Approx)?;
```

## Migration from 0.2.x

| Change | 0.2.x | 0.3.x |
|--------|-------|-------|
| Constructor | `Abp::new(i2c, delay, "PART-NR")` | `Abp::new(i2c, config)` — infallible |
| Config/parsing | `Abp::new` panics on bad part number | `AbpConfig::from_part_number(s)?` returns `Result` |
| Delay parameter | `Abp<I2C, D>` (two type params) | `Abp<I2C>` (one type param, no delay) |
| Temperature read | `sensor.pressure_and_temperature()` → `(f32, f32)` | `sensor.read_with_temperature(TemperatureResolution::Full)?` → `(f32, Option<f32>)` |
| Resolution choice | None — always 4 bytes | `TemperatureResolution::Approx` (3 bytes) or `Full` (4 bytes) |
| Pressure unit | Internal `conversion_factor: f32` | `PressureUnit` enum with `to_pa_factor()` and `Display` |
| `has_thermometer` | Hidden field | `sensor.config.has_thermometer` (public) |

Migration example:

```rust
// 0.2.x
let mut sensor = Abp::new(i2c, delay, "ABPDNNN030PG2A3");
let (p, t) = sensor.pressure_and_temperature()?;

// 0.3.x
let config = AbpConfig::from_part_number("ABPDNNN030PG2A3")?;
let mut sensor = Abp::new(i2c, config);
let (p, t) = sensor.read_with_temperature(TemperatureResolution::Full)?;
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
