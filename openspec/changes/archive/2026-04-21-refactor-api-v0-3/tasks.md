## 1. New public types

- [x] 1.1 Add `PressureUnit` enum (`Bar`, `Mbar`, `Kpa`, `Psi`) with `#[derive(Copy, Clone, Debug)]`
- [x] 1.2 Implement `PressureUnit::to_pa_factor(self) -> f32`
- [x] 1.3 Implement `core::fmt::Display` for `PressureUnit` (`"bar"`, `"mbar"`, `"kPa"`, `"psi"`)
- [x] 1.4 Add `ParseError` enum with variants: `NotAbpFamily`, `TooShort`, `InvalidPressureValue`, `InvalidUnit`, `InvalidType`, `InvalidAddress`, `InvalidTransferFunction`
- [x] 1.5 Add `TemperatureResolution` enum (`Approx`, `Full`) with `#[derive(Copy, Clone, Debug)]`
- [x] 1.6 Add `AbpConfig` public struct: `p_max: f32`, `p_min: f32`, `unit: PressureUnit`, `i2c_address: u8`, `has_thermometer: bool` — all fields public, `#[derive(Copy, Clone, Debug)]`

## 2. AbpConfig::from_part_number

- [x] 2.1 Implement `AbpConfig::from_part_number(part_nr: &str) -> Result<AbpConfig, ParseError>`
- [x] 2.2 Guard: return `Err(ParseError::TooShort)` if `part_nr.len() < 15`
- [x] 2.3 Guard: return `Err(ParseError::NotAbpFamily)` if `&part_nr[0..3] != "ABP"`
- [x] 2.4 Parse pressure value `&part_nr[7..10]` → `Err(ParseError::InvalidPressureValue)` on failure
- [x] 2.5 Match unit char `&part_nr[10..11]` → `PressureUnit`; `Err(ParseError::InvalidUnit)` on unknown
- [x] 2.6 Match type char `&part_nr[11..12]` → compute `p_min`; `Err(ParseError::InvalidType)` on unknown
- [x] 2.7 Match address char `&part_nr[12..13]` → `u8` address; `Err(ParseError::InvalidAddress)` for `A`/`S` and unknown
- [x] 2.8 Match transfer fn char `&part_nr[13..14]` → `has_thermometer: bool`; `Err(ParseError::InvalidTransferFunction)` on unknown

## 3. Abp struct and constructor

- [x] 3.1 Remove `D` type parameter from `Abp<I2C, D>` → `Abp<I2C>`
- [x] 3.2 Remove `delay: D` and `has_sleep: bool` fields from struct
- [x] 3.3 Replace inline fields (`p_max`, `p_min`, `o_max`, `o_min`, `conversion_factor`, `i2c_address`, `has_thermometer`) with `config: AbpConfig` and keep `o_max`/`o_min` as associated constants or local constants
- [x] 3.4 Rewrite `Abp::new(i2c: I2C, config: AbpConfig) -> Self` — infallible, no string parsing
- [x] 3.5 Update `impl<I2C> Abp<I2C> where I2C: I2c` — remove `D` bound everywhere

## 4. `read()` method

- [x] 4.1 Update `convert_pressure()` to use `self.config.unit.to_pa_factor()` and `self.config.p_max` / `p_min` instead of inline fields
- [x] 4.2 Verify `read()` compiles and tests pass (return type and logic unchanged)

## 5. `read_with_temperature()` method

- [x] 5.1 Add `read_with_temperature(&mut self, resolution: TemperatureResolution) -> Result<(f32, Option<f32>), AbpError<I2C::Error>>`
- [x] 5.2 Branch on `resolution`: read 3 bytes for `Approx`, 4 bytes for `Full`
- [x] 5.3 Map I2C error to `AbpError::I2c`
- [x] 5.4 Decode status bits and pressure counts from bytes 1–2 (same as `read()`)
- [x] 5.5 Assemble temperature counts: `Approx` → `(buffer[2] as u16) << 3`; `Full` → `((buffer[2] as u16) << 3) | ((buffer[3] as u16) >> 5)`
- [x] 5.6 Apply temperature formula: `(counts as f32 / 2047.0) * 200.0 - 50.0`
- [x] 5.7 Return `Some(temp)` if `self.config.has_thermometer`, else `None`
- [x] 5.8 Match status bits: Stale → `DataNotReady`, Command → `ErrorCommandMode`, Diagnostic → `ErrorDiagnosticState`

## 6. Remove obsolete code

- [x] 6.1 Remove `pressure_and_temperature()` method
- [x] 6.2 Remove `PressureUnit` orphan enum if it was re-added (should be the new one with methods)

## 7. Verification

- [x] 7.1 Run `cargo build` and fix all compilation errors
- [x] 7.2 Run `cargo clippy` and address warnings
- [x] 7.3 Update existing unit tests to use `AbpConfig` directly instead of part-number string
- [x] 7.4 Add unit tests for `PressureUnit::to_pa_factor()` (all four variants)
- [x] 7.5 Add unit tests for `AbpConfig::from_part_number()` — valid input and each error variant
- [x] 7.6 Add unit tests for `read_with_temperature` temperature assembly (Approx vs Full)
- [x] 7.7 Run `cargo test` — all tests pass

## 8. Documentation and versioning

- [x] 8.1 Bump `Cargo.toml` version to `0.3.0`
- [x] 8.2 Update crate-level doc comment with new usage example showing `from_part_number` and `read_with_temperature`
- [x] 8.3 Update `README.md` usage section and migration table (add 0.2 → 0.3 row)
