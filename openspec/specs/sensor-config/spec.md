# Spec: Sensor Configuration

## Purpose

Defines the `AbpConfig` struct and `PressureUnit` enum used to configure the ABP driver, including part number parsing, direct construction, and unit conversion.

## Requirements

### Requirement: Parse part number into validated sensor configuration
The driver SHALL expose `AbpConfig::from_part_number(part_nr: &str) -> Result<AbpConfig, ParseError>` that parses a Honeywell ABP part number string and returns a fully validated `AbpConfig` on success, or a `ParseError` variant on any invalid input. No panics SHALL occur inside this function.

Parsing extracts the 5-character pressure+type code from positions 7–11 (e.g. `"004BD"`, `"060MG"`, `"001GG"`) and matches it against the complete `PressureRange` enum allowlist. Any code not in the allowlist — including syntactically plausible but non-existent ranges — returns `Err(ParseError::InvalidPressureRange)`.

#### Scenario: Valid I2C part number parses successfully
- **WHEN** a well-formed Honeywell ABP I2C part number is passed (e.g. `"ABPDNNN030PG2A3"`)
- **THEN** `from_part_number` returns `Ok(config)` with correct `range`, `i2c_address`, and `has_thermometer` fields

#### Scenario: MPa gauge part number parses successfully
- **WHEN** `"ABPDNNN001GG2A3"` is passed to `from_part_number`
- **THEN** it returns `Ok(config)` with `config.range == PressureRange::Mpa1Gauge`

#### Scenario: Decimal bar part number parses successfully
- **WHEN** `"ABPDNNN1.6BD2A3"` is passed to `from_part_number`
- **THEN** it returns `Ok(config)` with `config.range == PressureRange::Bar1_6Differential`

#### Scenario: Non-ABP family string is rejected
- **WHEN** the first 3 characters of the string are not `"ABP"`
- **THEN** `from_part_number` returns `Err(ParseError::NotAbpFamily)`

#### Scenario: String too short is rejected
- **WHEN** the part number string is shorter than 15 characters
- **THEN** `from_part_number` returns `Err(ParseError::TooShort)`

#### Scenario: Unknown pressure+type code is rejected
- **WHEN** the 5-character code at positions 7–11 does not match any `PressureRange` variant (e.g. `"004MG"` — 4 mbar, not an orderable range)
- **THEN** `from_part_number` returns `Err(ParseError::InvalidPressureRange)`

#### Scenario: Gauge-only code with differential suffix is rejected
- **WHEN** a part number encodes a gauge-only range with a differential suffix (e.g. `"010BD"`)
- **THEN** `from_part_number` returns `Err(ParseError::InvalidPressureRange)`

#### Scenario: Analog/SPI output sensor is rejected
- **WHEN** the character at position 12 is `A` or `S`
- **THEN** `from_part_number` returns `Err(ParseError::InvalidAddress)`

#### Scenario: Unknown transfer function is rejected
- **WHEN** the character at position 13 is not one of `A`, `D`, `S`, `T`
- **THEN** `from_part_number` returns `Err(ParseError::InvalidTransferFunction)`

### Requirement: AbpConfig is directly constructable
`AbpConfig` SHALL be a public struct with all fields public, allowing direct construction without going through `from_part_number`. The struct SHALL contain `range: PressureRange`, `i2c_address: u8`, and `has_thermometer: bool`. The fields `p_max`, `p_min`, and `unit` are removed; pressure bounds are obtained via `config.range.p_min_pa()` and `config.range.p_max_pa()`.

#### Scenario: Direct construction with a valid PressureRange variant
- **WHEN** an `AbpConfig` is constructed directly with a `PressureRange` variant and valid `i2c_address` / `has_thermometer`
- **THEN** `Abp::new(i2c, config)` accepts it and functions identically to a parsed config

### Requirement: PressureUnit exposes Pa conversion factor and display
The `PressureUnit` enum SHALL expose `to_pa_factor(self) -> f32` returning the multiplier to convert a native-unit pressure value to Pascals. It SHALL implement `core::fmt::Display`. The enum SHALL include variants: `Bar`, `Mbar`, `Kpa`, `Psi`, and `Mpa`.

| Variant | `to_pa_factor()` | Display |
|---------|-----------------|---------|
| `Bar`   | `100_000.0`     | `"bar"` |
| `Mbar`  | `100.0`         | `"mbar"` |
| `Kpa`   | `1_000.0`       | `"kPa"` |
| `Psi`   | `6_894.757_3`   | `"psi"` |
| `Mpa`   | `1_000_000.0`   | `"MPa"` |

#### Scenario: PSI factor is correct
- **WHEN** `PressureUnit::Psi.to_pa_factor()` is called
- **THEN** the returned value equals `6_894.757_3` within float tolerance

#### Scenario: MPa factor is correct
- **WHEN** `PressureUnit::Mpa.to_pa_factor()` is called
- **THEN** the returned value equals `1_000_000.0`

#### Scenario: Display formats correctly
- **WHEN** `PressureUnit::Mpa` is formatted via `Display`
- **THEN** the output string is `"MPa"`

#### Scenario: Native-unit round-trip
- **WHEN** a Pa value is divided by `unit.to_pa_factor()` and the result multiplied back
- **THEN** the original Pa value is recovered within floating-point tolerance
