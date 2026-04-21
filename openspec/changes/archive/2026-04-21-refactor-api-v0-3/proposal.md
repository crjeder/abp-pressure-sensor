## Why

The 0.2 API still has rough edges inherited from 0.1: construction panics instead of returning errors, the delay type parameter is unused dead weight, part-number parsing is inseparable from I2C wiring, and the two read methods (`read` / `pressure_and_temperature`) obscure the real hardware choice (how many I2C bytes to clock). This change produces a clean, ergonomic 0.3 API designed from first principles against the Honeywell datasheet.

## What Changes

- **BREAKING** Add `AbpConfig` public struct; `from_part_number(&str) -> Result<AbpConfig, ParseError>` replaces the part-number string argument on `new()`
- **BREAKING** Add `ParseError` enum — all panics in part-number parsing replaced with `Result` variants
- **BREAKING** Add `PressureUnit` enum (`Bar`, `Mbar`, `Kpa`, `Psi`) with `to_pa_factor() -> f32` and `core::fmt::Display`; stored on `AbpConfig` instead of a bare `f32` conversion factor
- **BREAKING** Drop `D` (delay) type parameter from `Abp<I2C, D>` → `Abp<I2C>`; `delay` field and `has_sleep` removed (sleep mode not yet implemented; delay can be added back as a method parameter when needed)
- **BREAKING** `Abp::new(i2c, config: AbpConfig)` — infallible constructor, no string parsing
- **BREAKING** Remove `pressure_and_temperature()` method
- **BREAKING** `read()` returns `Result<f32, AbpError<I2C::Error>>` — unchanged signature, 2-byte I2C read (fastest path)
- **BREAKING** Add `read_with_temperature(resolution: TemperatureResolution) -> Result<(f32, Option<f32>), AbpError<I2C::Error>>`
- **BREAKING** Add `TemperatureResolution` enum (`Approx` = 3-byte / 8-bit, `Full` = 4-byte / 11-bit)
- Temperature `Option<f32>` is `Some` only when `config.has_thermometer` is true
- Bump crate to `0.3.0`

## Capabilities

### New Capabilities

- `sensor-config`: Parse a Honeywell part number string into a validated `AbpConfig`; expose `PressureUnit` with Pa conversion and display
- `temperature-resolution-read`: Read pressure + optional temperature with caller-chosen resolution (`Approx` / `Full`)

### Modified Capabilities

- `pressure-read`: `read()` signature unchanged but construction path changes; `AbpError` type unchanged. Requirement update: construction is now fallible via `from_part_number`, not panicking.

## Impact

- `src/lib.rs`: full API rewrite; all public types affected
- Callers must update: add `from_part_number` call, handle `ParseError`, remove delay type, switch `pressure_and_temperature()` to `read_with_temperature()`
- No new crate dependencies
- Crate version: `0.2.0` → `0.3.0`
