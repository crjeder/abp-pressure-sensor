# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-04-21

### Added

- `AbpConfig` struct — validated sensor configuration with public fields (`p_max`, `p_min`, `unit`, `i2c_address`, `has_thermometer`); can be constructed directly or via `from_part_number`
- `AbpConfig::from_part_number(part_nr: &str) -> Result<AbpConfig, ParseError>` — fallible part-number parser; replaces the panicking logic that was embedded in `new()`
- `ParseError` enum — fine-grained parse errors: `TooShort`, `NotAbpFamily`, `InvalidPressureValue`, `InvalidUnit`, `InvalidType`, `InvalidAddress`, `InvalidTransferFunction`
- `PressureUnit` enum (`Bar`, `Mbar`, `Kpa`, `Psi`) with `to_pa_factor() -> f32` and `impl Display` (`"bar"`, `"mbar"`, `"kPa"`, `"psi"`)
- `TemperatureResolution` enum (`Approx`, `Full`) for caller-controlled read width
- `Abp::read_with_temperature(resolution: TemperatureResolution) -> Result<(f32, Option<f32>), AbpError<E>>` — 3-byte read for 8-bit (~0.8 °C) or 4-byte read for 11-bit (~0.1 °C) temperature; returns `None` when sensor has no thermometer

### Changed

- `Abp::new(i2c, config: AbpConfig) -> Self` — constructor is now infallible; part-number parsing has moved to `AbpConfig::from_part_number`
- `Abp<I2C>` — delay type parameter `D` removed; no delay is needed for the read-only I2C protocol
- `read()` — now uses `config.unit.to_pa_factor()` for the Pa conversion; behaviour and return type unchanged
- `config` field on `Abp` is now public, giving callers direct access to `sensor.config.unit`, `sensor.config.has_thermometer`, etc.

### Removed

- `pressure_and_temperature() -> Result<(f32, f32), …>` — replaced by `read_with_temperature(TemperatureResolution)` which returns `(f32, Option<f32>)` and exposes resolution choice. **Migration:** replace `sensor.pressure_and_temperature()?` with `sensor.read_with_temperature(TemperatureResolution::Full)?`; temperature moves from `(f32, f32)` to `(f32, Option<f32>)`

## [0.2.0] - 2026-04-20

### Added

- `AbpError::DataNotReady` — replaces `nb::Error::WouldBlock` for stale sensor data
- `AbpError::ErrorCommandMode` and `AbpError::ErrorDiagnosticState` — explicit error variants instead of the previous `quick-error` macro definitions
- `pressure_and_temperature() -> Result<(f32, f32), AbpError<E>>` — returns decoded temperature alongside pressure (temperature was previously ignored in the 4-byte read path)
- 8 unit tests covering the pressure conversion formula and all four status-bit decode cases

### Changed

- Updated `embedded-hal` dependency from 0.2.7 to **1.0.0**; I2C bound changed from `blocking::i2c::Read` to `embedded_hal::i2c::I2c`; delay bound changed from `blocking::delay::DelayMs` to `DelayNs`
- `read()` return type changed from `nb::Result<f32, AbpError>` to `Result<f32, AbpError<I2C::Error>>`
- Fixed inverted `convert_pressure()` formula — was computing `(o_max - o_min) / (p_max - p_min)` (wrong direction); now correctly `(p_max - p_min) / (o_max - o_min)`
- Fixed systematic off-by-one errors in part-number slice indices (original `substring` crate calls used wrong positions)

### Fixed

- Typo in error type name: `ApbError` → `AbpError`

### Removed

- Dependencies on `nb`, `quick-error`, and `substring` crates

## [0.1.0] - 2022-03-06

### Added

- Initial `no_std` driver for Honeywell ABP series I2C pressure sensors
- `Abp<I2C, D>` struct generic over an embedded-hal 0.2 I2C and delay implementation
- `Abp::new(i2c, delay, part_nr: &str)` — constructs driver by parsing the Honeywell part number string
- `read() -> nb::Result<f32, AbpError>` — 2-byte I2C read returning pressure in Pascals
- `bitmatch`-based bit decoding of the 2-bit status field and 14-bit pressure count
- Part-number parsing for pressure range, unit, sensor type, I2C address, and thermometer presence

[Unreleased]: https://github.com/crjeder/abp-pressure-sensor/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/crjeder/abp-pressure-sensor/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/crjeder/abp-pressure-sensor/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/crjeder/abp-pressure-sensor/releases/tag/v0.1.0
