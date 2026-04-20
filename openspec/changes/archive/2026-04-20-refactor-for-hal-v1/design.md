## Context

The driver is a single-file `no_std` crate (`src/lib.rs`) wrapping the Honeywell ABP I2C pressure sensor. It currently depends on `embedded-hal 0.2.7`, which exposes blocking I2C as `embedded_hal::blocking::i2c::Read` and blocking delay as `embedded_hal::blocking::delay::DelayMs<u16>`. `embedded-hal 1.0` reorganised these under `embedded_hal::i2c::I2c` (a combined read/write/read-write trait) and `embedded_hal::delay::DelayMs` (no type parameter). The 0.2 and 1.0 APIs are not compatible; a semver bump is required.

There are also two pre-existing correctness bugs being fixed in the same pass:
- `convert_pressure()` has the linear mapping inverted (divides span by span, not span by counts).
- `pressure_and_temperature()` decodes temperature but discards it.

## Goals / Non-Goals

**Goals:**
- Compile cleanly on stable Rust with `embedded-hal 1.0`
- Preserve the existing public API surface (`Abp::new`, `read`, `pressure_and_temperature`) with updated signatures where HAL 1.0 requires it
- Fix `convert_pressure()` formula
- Return `(f32, f32)` (pressure, temperature) from `pressure_and_temperature()`
- Remove unused dependencies (`nb`, `quick-error`, optionally `substring`)

**Non-Goals:**
- SPI variant support
- Async (`embedded-hal-async`) support — future change
- `defmt` or `log` integration
- Changing the part-number parsing logic beyond what is required by the refactor

## Decisions

### D1: Adopt `embedded_hal::i2c::I2c` as the single I2C bound

In HAL 1.0 there is no separate `Read` trait; `I2c` covers read, write, and write-read. Bound as `I2C: embedded_hal::i2c::I2c`. The error type is `I2C::Error`.

*Alternative considered:* keep a `Read`-only bound via `embedded_hal_nb::i2c::I2c`. Rejected — `embedded_hal_nb` is for non-blocking use; the driver is inherently blocking.

### D2: Drop `nb` entirely

HAL 1.0 blocking operations return `Result<_, E>` directly. The `Stale` / `WouldBlock` case (sensor data not yet ready) is returned as a distinct error variant in `AbpError`, not as `nb::Error::WouldBlock`.

*Alternative:* keep `nb` and wrap manually. Rejected — adds a dependency with no benefit once the HAL no longer requires it.

### D3: `DelayMs` without type parameter

`embedded_hal::delay::DelayMs` in v1.0 takes `u32` (via `delay_ms(ms: u32)`). Replace `D: DelayMs<u16>` with `D: embedded_hal::delay::DelayMs`.

### D4: Replace `substring` with standard slice indexing

`part_nr.substring(a, b)` is equivalent to `&part_nr[a..=b]` (byte-indexed on ASCII-safe part numbers). Remove the `substring` crate dependency.

### D5: Fix `convert_pressure()` formula

The datasheet linear mapping is:

```
pressure = (p_max - p_min) / (o_max - o_min) * (output - o_min) + p_min
```

Current code has numerator and denominator swapped and mixes pressure-range values with output counts. Corrected formula uses the above.

### D6: `pressure_and_temperature()` returns `(f32, f32)`

Change return type from `Result<f32, E>` to `Result<(f32, f32), I2C::Error>`. The second element is the temperature in °C using the existing `convert_temperature()` helper.

## Risks / Trade-offs

- **User breaking change** → Clearly document in CHANGELOG and README. Bump crate major version to 0.2.0 (not yet 1.0 since the API itself is still evolving).
- **`bitmatch` crate compatibility** → `bitmatch 0.1.1` has no `embedded-hal` dependency; it is unaffected by the upgrade. Risk: low.
- **Correctness of `convert_pressure()` fix** → The fix introduces a behaviour change for existing users. Old outputs were wrong; new outputs will match the datasheet. Risk: low but must be documented.

## Migration Plan

1. Bump `embedded-hal` in `Cargo.toml` to `"1.0"`.
2. Remove `nb`, `quick-error`, `substring` from `Cargo.toml`.
3. Update `src/lib.rs` imports, trait bounds, and method bodies.
4. Run `cargo build` and `cargo test` to verify.
5. Bump crate version in `Cargo.toml` to `0.2.0`.
6. Update `README.md` with new usage example and migration note.

## Open Questions

- Should `AbpError::Stale` replace the old `WouldBlock` semantic, or should we expose a dedicated `DataNotReady` variant? (Recommendation: `DataNotReady` is clearer.)
- Is `delay` still needed in the struct after removing sleep-mode support from scope? (Current code stores it but never uses it in `read()`.) Leave for now; remove in a follow-up.
