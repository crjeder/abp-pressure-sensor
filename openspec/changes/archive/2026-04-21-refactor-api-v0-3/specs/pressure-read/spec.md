## MODIFIED Requirements

### Requirement: Read pressure via embedded-hal v1.0 I2c trait
The driver SHALL expose a `read(&mut self) -> Result<f32, AbpError<I2C::Error>>` method on `Abp<I2C>` where `I2C: embedded_hal::i2c::I2c`. The returned value SHALL be pressure in Pascals computed by applying the datasheet linear mapping to the 14-bit sensor output count. The driver instance SHALL be created via `Abp::new(i2c, config: AbpConfig)` where `AbpConfig` is obtained from `AbpConfig::from_part_number(&str)` or constructed directly; no delay type parameter is present.

#### Scenario: Valid sensor data
- **WHEN** the sensor returns a 2-byte response with status bits `0b00` (Valid)
- **THEN** `read()` returns `Ok(pressure_in_pa)` where the value is derived from the correct datasheet formula

#### Scenario: Stale data
- **WHEN** the sensor returns status bits `0b10` (Stale)
- **THEN** `read()` returns `Err(AbpError::DataNotReady)`

#### Scenario: Command mode
- **WHEN** the sensor returns status bits `0b01` (Command mode)
- **THEN** `read()` returns `Err(AbpError::ErrorCommandMode)`

#### Scenario: Diagnostic condition
- **WHEN** the sensor returns status bits `0b11` (Diagnostic)
- **THEN** `read()` returns `Err(AbpError::ErrorDiagnosticState)`

#### Scenario: I2C bus error
- **WHEN** the underlying I2C `read()` call returns an error
- **THEN** `read()` returns `Err(AbpError::I2c(e))` wrapping the HAL error

## REMOVED Requirements

### Requirement: Pressure conversion formula correctness
**Reason**: The formula is unchanged and correct since 0.2; it is now an internal implementation detail of `convert_pressure()` rather than a separately specced requirement. Removing to avoid duplication with the implementation.
**Migration**: No action needed — callers are unaffected. The formula `(p_max - p_min) / (o_max - o_min) * (reading - o_min) + p_min` multiplied by `unit.to_pa_factor()` remains the implementation.
