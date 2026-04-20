## Why

The driver targets `embedded-hal` 0.2.7, which uses the deprecated `blocking::*` module hierarchy and `nb`-based error types. `embedded-hal` 1.0 (released late 2023) is now the stable baseline for the embedded-Rust ecosystem and all modern HAL implementations; staying on 0.2 blocks users from integrating this driver with any current board support crate.

## What Changes

- **BREAKING** Replace `embedded-hal 0.2.7` dependency with `embedded-hal 1.0`
- **BREAKING** Update `I2C` trait bound from `hal::blocking::i2c::Read` to `hal::i2c::I2c`
- **BREAKING** Update `D` (delay) trait bound from `hal::blocking::delay::DelayMs<u16>` to `hal::delay::DelayMs`
- **BREAKING** Update `read()` return type: drop `nb::Result` wrapper; use `Result<f32, E>` with the I2c associated error type directly
- Update `pressure_and_temperature()` to correctly thread the HAL v1 error type
- Fix the `convert_pressure()` formula (currently maps in the wrong direction)
- Remove `nb` dependency (no longer needed for blocking reads in HAL v1)
- Remove `quick-error` dependency (unused)
- Replace `substring` crate usage with standard `&str` slicing (reduce dependencies)

## Capabilities

### New Capabilities

- `pressure-read`: Read a single pressure value over I2C using embedded-hal v1.0 traits — returns `Result<f32, E>` where `E` is the I2c error type
- `pressure-temperature-read`: Read both pressure and temperature in one 4-byte transaction; return both values as `(f32, f32)`

### Modified Capabilities

*(No existing specs — this is a new capability baseline.)*

## Impact

- `Cargo.toml`: `embedded-hal` bumped to `1.0`, `nb` removed, `quick-error` removed, optionally `substring` removed
- `src/lib.rs`: all trait bounds, imports, and method signatures updated
- Users must update their `embedded-hal` impl crate to a v1-compatible version (e.g. `rppal` ≥ 0.18, `stm32f4xx-hal` ≥ 0.21)
