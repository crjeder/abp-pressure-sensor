## ADDED Requirements

### Requirement: Parse part number into validated sensor configuration
The driver SHALL expose `AbpConfig::from_part_number(part_nr: &str) -> Result<AbpConfig, ParseError>` that parses a Honeywell ABP part number string and returns a fully validated `AbpConfig` on success, or a `ParseError` variant on any invalid input. No panics SHALL occur inside this function.

#### Scenario: Valid I2C part number parses successfully
- **WHEN** a well-formed Honeywell ABP I2C part number is passed (e.g. `"ABPDNNN030PG2A3"`)
- **THEN** `from_part_number` returns `Ok(config)` with correct `p_max`, `p_min`, `unit`, `i2c_address`, and `has_thermometer` fields

#### Scenario: Non-ABP family string is rejected
- **WHEN** the first 3 characters of the string are not `"ABP"`
- **THEN** `from_part_number` returns `Err(ParseError::NotAbpFamily)`

#### Scenario: String too short is rejected
- **WHEN** the part number string is shorter than 15 characters
- **THEN** `from_part_number` returns `Err(ParseError::TooShort)`

#### Scenario: Invalid pressure value is rejected
- **WHEN** characters at positions 7–9 cannot be parsed as a decimal integer
- **THEN** `from_part_number` returns `Err(ParseError::InvalidPressureValue)`

#### Scenario: Unknown pressure unit is rejected
- **WHEN** the character at position 10 is not one of `M`, `B`, `K`, `P`
- **THEN** `from_part_number` returns `Err(ParseError::InvalidUnit)`

#### Scenario: Unknown sensor type is rejected
- **WHEN** the character at position 11 is not `D` or `G`
- **THEN** `from_part_number` returns `Err(ParseError::InvalidType)`

#### Scenario: Analog/SPI output sensor is rejected
- **WHEN** the character at position 12 is `A` or `S`
- **THEN** `from_part_number` returns `Err(ParseError::InvalidAddress)`

#### Scenario: Unknown transfer function is rejected
- **WHEN** the character at position 13 is not one of `A`, `D`, `S`, `T`
- **THEN** `from_part_number` returns `Err(ParseError::InvalidTransferFunction)`

### Requirement: AbpConfig is directly constructable
`AbpConfig` SHALL be a public struct with all fields public, allowing direct construction without going through `from_part_number`. This enables testing and use cases where sensor parameters are known at compile time.

#### Scenario: Direct construction bypasses parsing
- **WHEN** an `AbpConfig` is constructed directly with known field values
- **THEN** `Abp::new(i2c, config)` accepts it and functions identically to a parsed config

### Requirement: PressureUnit exposes Pa conversion factor and display
The `PressureUnit` enum SHALL expose `to_pa_factor(self) -> f32` returning the multiplier to convert a native-unit pressure value to Pascals. It SHALL implement `core::fmt::Display` returning lowercase unit strings (`"bar"`, `"mbar"`, `"kPa"`, `"psi"`).

#### Scenario: PSI factor is correct
- **WHEN** `PressureUnit::Psi.to_pa_factor()` is called
- **THEN** the returned value equals `6_894.757_3` within float tolerance

#### Scenario: kPa factor is correct
- **WHEN** `PressureUnit::Kpa.to_pa_factor()` is called
- **THEN** the returned value equals `1_000.0`

#### Scenario: Display formats correctly
- **WHEN** `PressureUnit::Psi` is formatted via `Display`
- **THEN** the output string is `"psi"`

#### Scenario: Native-unit round-trip
- **WHEN** a Pa value is divided by `unit.to_pa_factor()` and the result multiplied back
- **THEN** the original Pa value is recovered within floating-point tolerance
