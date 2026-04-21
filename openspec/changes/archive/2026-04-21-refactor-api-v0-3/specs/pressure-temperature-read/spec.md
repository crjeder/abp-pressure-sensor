## REMOVED Requirements

### Requirement: Read pressure and temperature together
**Reason**: Replaced by `read_with_temperature(TemperatureResolution)` in the `temperature-resolution-read` capability. The new method exposes the same hardware transaction but gives the caller control over 3-byte (8-bit, `Approx`) vs 4-byte (11-bit, `Full`) resolution, and correctly returns `Option<f32>` for sensors without a thermometer.
**Migration**: Replace `sensor.pressure_and_temperature()` with `sensor.read_with_temperature(TemperatureResolution::Full)`. The pressure value is unchanged; temperature moves from the second element of a `(f32, f32)` tuple to `Some(f32)` inside `(f32, Option<f32>)`.
