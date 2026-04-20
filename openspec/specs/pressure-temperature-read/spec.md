# Spec: Pressure and Temperature Read

## Purpose

Defines the behaviour of the `pressure_and_temperature()` method on the ABP driver for reading pressure and temperature together in a single 4-byte I2C transaction via the embedded-hal v1.0 I2C trait.

## Requirements

### Requirement: Read pressure and temperature together
The driver SHALL expose a `pressure_and_temperature(&mut self) -> Result<(f32, f32), AbpError<I2C::Error>>` method on `Abp<I2C, D>` where `I2C: embedded_hal::i2c::I2c`. The method SHALL issue a 4-byte I2C read and return `(pressure_pa, temperature_celsius)`.

#### Scenario: Sensor with thermometer returns both values
- **WHEN** the sensor returns a valid 4-byte response with status bits `0b00`
- **THEN** `pressure_and_temperature()` returns `Ok((pressure_pa, temperature_celsius))` where temperature is decoded from bits 8..18 of bytes 2–3

#### Scenario: Temperature conversion formula
- **WHEN** the raw 11-bit temperature count is `t`
- **THEN** the temperature in °C SHALL equal `(t / 2047.0) * 200.0 - 50.0`

#### Scenario: Stale data on combined read
- **WHEN** the sensor returns status bits `0b10` in the 4-byte response
- **THEN** `pressure_and_temperature()` returns `Err(AbpError::DataNotReady)`

#### Scenario: I2C bus error on combined read
- **WHEN** the underlying I2C `read()` call fails
- **THEN** `pressure_and_temperature()` returns `Err(AbpError::I2c(e))`
