## ADDED Requirements

### Requirement: Read pressure and optional temperature with caller-chosen resolution
The driver SHALL expose `read_with_temperature(&mut self, resolution: TemperatureResolution) -> Result<(f32, Option<f32>), AbpError<I2C::Error>>` on `Abp<I2C>`. The method SHALL issue a 3-byte I2C read for `Approx` or a 4-byte read for `Full`, and return `(pressure_pa, Some(temperature_celsius))` when `config.has_thermometer` is true, or `(pressure_pa, None)` when false.

#### Scenario: Full resolution on thermometer sensor
- **WHEN** `resolution` is `TemperatureResolution::Full` and `config.has_thermometer` is true
- **THEN** the method issues a 4-byte I2C read and returns `Ok((pressure_pa, Some(temp_c)))` with 11-bit temperature precision (~0.1 °C)

#### Scenario: Approx resolution on thermometer sensor
- **WHEN** `resolution` is `TemperatureResolution::Approx` and `config.has_thermometer` is true
- **THEN** the method issues a 3-byte I2C read and returns `Ok((pressure_pa, Some(temp_c)))` with 8-bit temperature precision (~0.8 °C)

#### Scenario: No thermometer — temperature is None regardless of resolution
- **WHEN** `config.has_thermometer` is false
- **THEN** the method returns `Ok((pressure_pa, None))` regardless of the `resolution` argument

#### Scenario: Stale data
- **WHEN** the sensor returns status bits `0b10`
- **THEN** `read_with_temperature` returns `Err(AbpError::DataNotReady)`

#### Scenario: I2C bus error
- **WHEN** the underlying I2C `read()` call fails
- **THEN** `read_with_temperature` returns `Err(AbpError::I2c(e))`

### Requirement: Temperature assembly uses unified 11-bit formula
Regardless of resolution, the temperature count SHALL be assembled as an 11-bit value before applying the conversion formula, with the low 3 bits set to zero for `Approx`:

```
Approx: counts = (byte3 as u16) << 3
Full:   counts = ((byte3 as u16) << 3) | ((byte4 as u16) >> 5)
temp_c  = (counts as f32 / 2047.0) * 200.0 - 50.0
```

#### Scenario: Approx and Full agree at low-3-bit boundary
- **WHEN** the sensor temperature lower 3 bits are all zero
- **THEN** `Approx` and `Full` return identical temperature values

#### Scenario: Full captures sub-degree precision
- **WHEN** the sensor temperature lower 3 bits are non-zero
- **THEN** `Full` returns a temperature that differs from `Approx` by less than 0.8 °C
