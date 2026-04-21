## Context

The 0.2 driver compiles and works but carries three design debts: (1) `new()` panics on bad part-number strings — unacceptable for bare-metal code that can't recover from a panic; (2) the delay type parameter `D` is unused, burdening every generic instantiation and every `impl` block for no reason; (3) the two read methods (`read` / `pressure_and_temperature`) do not map onto the hardware's actual capability — the sensor always delivers up to 4 bytes; the master controls transaction length by sending NACK, and the number of bytes determines temperature resolution (8-bit or 11-bit), not just presence/absence.

A second insight from the datasheet: pressure unit is a first-class property of the sensor (parsed from the part number), not an internal float. Exposing `PressureUnit` with a `to_pa_factor()` method and `Display` lets callers do unit-aware display without raw arithmetic.

## Goals / Non-Goals

**Goals:**
- Fallible part-number parsing: `AbpConfig::from_part_number(&str) -> Result<AbpConfig, ParseError>`
- Zero panics in library code; all error conditions return `Result`
- Drop `D` type parameter; `Abp<I2C>` only
- Public `PressureUnit` enum with `to_pa_factor()` and `core::fmt::Display`
- `AbpConfig` is a fully public, `Copy`-able struct — constructable directly for tests or known sensors
- `read()` stays 2-byte / fast path
- `read_with_temperature(TemperatureResolution)` unifies the old `pressure_and_temperature()` and adds `Approx` (3-byte / 8-bit) and `Full` (4-byte / 11-bit) modes
- Temperature return is `Option<f32>` — `Some` only when `config.has_thermometer`

**Non-Goals:**
- Sleep mode / wakeup (delay stays out of the struct; add back when sleep is implemented)
- Async (`embedded-hal-async`) support
- `defmt` derive support (can be added as a feature flag later)
- SPI variant

## Decisions

### D1: `AbpConfig` as a public plain struct, not an opaque builder

`AbpConfig` holds all sensor parameters as public fields. Users can construct it directly without going through `from_part_number` — useful in tests and when sensor parameters are known at compile time. `from_part_number` is a convenience parser, not the only path.

*Alternative:* opaque newtype with only `from_part_number`. Rejected — prevents direct construction and makes tests harder without any safety benefit.

### D2: `PressureUnit` owns the conversion factor

`PressureUnit::to_pa_factor()` replaces the bare `f32 conversion_factor` field. This means callers can label readings (via `Display`) and the factor is always consistent with the unit. `AbpConfig` stores `unit: PressureUnit`; `convert_pressure()` calls `config.unit.to_pa_factor()` internally.

### D3: `ParseError` as a flat enum, no payload

```rust
pub enum ParseError {
    NotAbpFamily,
    InvalidPressureValue,
    InvalidUnit,
    InvalidType,
    InvalidAddress,
    InvalidTransferFunction,
    TooShort,
}
```

Each variant maps to exactly one panic site in 0.2. No payload on variants — `no_std` without `alloc`, no heap for strings. Variants are specific enough to identify the problem at startup.

### D4: `TemperatureResolution` enum not `bool`

A two-variant enum (`Approx`, `Full`) is a named boolean — same runtime cost, self-documenting at every call site. `sensor.read_with_temperature(TemperatureResolution::Full)` vs `sensor.read_with_temperature(true)`.

### D5: Unified temperature assembly formula

The 11-bit temperature value is assembled identically for both modes:

```
counts = (byte3 as u16) << 3          // Approx: T[10:3], low 3 bits = 0
counts = (byte3 as u16) << 3
       | (byte4 as u16) >> 5          // Full:   T[10:0]
temp_c = (counts as f32 / 2047.0) * 200.0 - 50.0
```

The formula is the same; `Approx` mode simply never ORs in the low 3 bits. The existing `bitmatch` decode for 4 bytes already handles `Full`; `Approx` needs a simpler 3-byte decode.

### D6: `has_sleep` removed, `has_thermometer` kept

`has_sleep` has no observable effect on any method — removing it simplifies `AbpConfig`. `has_thermometer` gates the `Option<f32>` in `read_with_temperature` — it must stay.

## Risks / Trade-offs

- **Third breaking change in a row** → Acceptable at 0.x; API is being designed to last before 1.0. Clearly communicate in CHANGELOG.
- **`from_part_number` takes `&str` not `&'static str`** → Better ergonomics, but callers with `&'static str` constants still work. No risk.
- **Direct `AbpConfig` construction bypasses validation** → Intentional. Library code trusts the caller who bypasses the parser. Document clearly.
- **`bitmatch` for 3-byte decode** → `bitmatch` works on fixed-length buffers; `Approx` path uses a `[u8; 3]` buffer and a simpler decode (byte-shift only, no `bitmatch` needed for temperature bits). Low risk.

## Migration Plan

1. Add `PressureUnit`, `AbpConfig`, `ParseError`, `TemperatureResolution` types
2. Rewrite `Abp<I2C>` (drop `D`), update `new(i2c, config)`, update `read()`
3. Add `read_with_temperature(resolution)`
4. Remove `pressure_and_temperature()`
5. Update `Cargo.toml` to `0.3.0`
6. Update README and doc examples

## Open Questions

*(none — all decisions made during design exploration)*
