## 1. Dependency updates

- [x] 1.1 Bump `embedded-hal` to `"1.0"` in `Cargo.toml`
- [x] 1.2 Remove `nb` from `Cargo.toml`
- [x] 1.3 Remove `quick-error` from `Cargo.toml`
- [x] 1.4 Remove `substring` from `Cargo.toml`

## 2. Imports and type aliases

- [x] 2.1 Replace `use embedded_hal as hal; use hal::blocking::{i2c, delay::DelayMs};` with `use embedded_hal::{i2c::I2c, delay::DelayNs};`
- [x] 2.2 Remove `use nb::{Error::{Other, WouldBlock}};`
- [x] 2.3 Remove `use substring::Substring;`
- [x] 2.4 Remove the `type I2cError = embedded_hal::blocking::i2c::Read::Error;` alias

## 3. Error type

- [x] 3.1 Rename `ApbError::Other(E)` variant to `AbpError::I2c(E)` for clarity (also fixed typo Apb→Abp)
- [x] 3.2 Add `AbpError::DataNotReady` variant (replaces the old `nb::Error::WouldBlock` case)

## 4. Struct and trait bounds

- [x] 4.1 Update `Abp<I2C, D>` where clause: `I2C: i2c::Read` → `I2C: I2c`
- [x] 4.2 Update delay bound: `D: DelayMs<u16>` → `D: DelayNs`
- [x] 4.3 Update `impl<I2C, D, E>` — remove the unused `E` type parameter; use `I2C::Error` directly

## 5. Part-number parsing (`new()`)

- [x] 5.1 Replace all `part_nr.substring(a, b)` calls with correct 0-indexed `&part_nr[a..b]` slice syntax (also fixed systematic off-by-one errors in the original indices)

## 6. `read()` method

- [x] 6.1 Change return type from `nb::Result<f32, nb::Error<ApbError<I2cError>>>` to `Result<f32, AbpError<I2C::Error>>`
- [x] 6.2 Replace `self.i2c.read(...)` call with HAL v1 `I2c::read()` signature; map I2C errors to `AbpError::I2c`
- [x] 6.3 Replace `Err(nb::Error::WouldBlock)` with `Err(AbpError::DataNotReady)` in the `Stale` branch
- [x] 6.4 Replace `Err(Other(...))` with `Err(AbpError::...)` direct variants

## 7. `pressure_and_temperature()` method

- [x] 7.1 Change return type to `Result<(f32, f32), AbpError<I2C::Error>>`
- [x] 7.2 Update `self.i2c.read(...)` to HAL v1 signature; map errors to `AbpError::I2c`
- [x] 7.3 Compute temperature via `convert_temperature(output.temperature as f32)` and include it in the `Ok` tuple
- [x] 7.4 Add status matching (Stale → `DataNotReady`, etc.) mirroring `read()`

## 8. Fix `convert_pressure()` formula

- [x] 8.1 Replace the inverted formula with the correct datasheet mapping (also multiplies by `conversion_factor` to yield Pascals):
  `(p_max - p_min) / f32::from(o_max - o_min) * (reading - f32::from(o_min)) + p_min`

## 9. Verification

- [x] 9.1 Run `cargo build` and fix any remaining compilation errors
- [x] 9.2 Run `cargo clippy` and address warnings
- [x] 9.3 Add unit tests for `convert_pressure()` mid-scale and boundary values
- [x] 9.4 Add unit tests for `decode_pressure()` covering all four status values
- [x] 9.5 Run `cargo test` to confirm all tests pass

## 10. Documentation and versioning

- [x] 10.1 Bump crate version in `Cargo.toml` to `0.2.0`
- [x] 10.2 Update the `README.md` usage example to reflect HAL v1 imports and the new `pressure_and_temperature()` return type
- [x] 10.3 Add a short migration note in `README.md` for users upgrading from 0.1.x
