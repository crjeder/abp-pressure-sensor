# Spec: Pressure Read

## Purpose

Defines the behaviour of the `read()` method on the ABP driver for reading a single pressure value via the embedded-hal v1.0 I2C trait.

## Requirements

### Requirement: Read pressure via embedded-hal v1.0 I2c trait
The driver SHALL expose a `read(&mut self) -> Result<f32, AbpError<I2C::Error>>` method on `Abp<I2C, D>` where `I2C: embedded_hal::i2c::I2c`. The returned value SHALL be pressure in Pascals computed by applying the datasheet linear mapping to the 14-bit sensor output count.

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

### Requirement: Pressure conversion formula correctness
The driver SHALL convert raw 14-bit output counts to Pascals using the formula:

```
pressure_pa = (p_max - p_min) / (o_max - o_min) * (reading - o_min) + p_min
```

where `p_max` and `p_min` are the sensor's pressure range in Pa, and `o_max = 0x3999`, `o_min = 0x0666`.

#### Scenario: Mid-scale reading maps to mid-scale pressure
- **WHEN** the 14-bit reading equals `(o_max + o_min) / 2`
- **THEN** the returned pressure equals `(p_max + p_min) / 2` within floating-point tolerance
