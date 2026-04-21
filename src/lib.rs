//! ABP series pressure sensor driver
//!
//! A platform-agnostic `no_std` driver for Honeywell ABP series I2C pressure sensors,
//! built on [`embedded-hal`](https://github.com/rust-embedded/embedded-hal) 1.0 traits.
//!
//! # Usage
//!
//! Parse the Honeywell part number to obtain an [`AbpConfig`], then construct [`Abp`]:
//!
//! ```rust,ignore
//! use abp_pressure_sensor::{Abp, AbpConfig, TemperatureResolution};
//!
//! let config = AbpConfig::from_part_number("ABPDNNN030PG2A3").unwrap();
//! let mut sensor = Abp::new(i2c, config);
//!
//! // Fast 2-byte read — pressure only
//! let pressure_pa = sensor.read()?;
//!
//! // Convert to native unit for display
//! let native = pressure_pa / config.unit.to_pa_factor();
//! // e.g. format!("{:.1} {}", native, config.unit) → "30.0 psi"
//!
//! // 4-byte read — pressure + 11-bit temperature (full precision)
//! let (pressure_pa, temp_opt) = sensor.read_with_temperature(TemperatureResolution::Full)?;
//! if let Some(temp_c) = temp_opt {
//!     // sensor has thermometer
//! }
//! ```
//!
//! # References
//!
//! - [Datasheet](https://prod-edam.honeywell.com/content/dam/honeywell-edam/sps/siot/de-de/products/sensors/pressure-sensors/board-mount-pressure-sensors/basic-abp-series/documents/sps-siot-basic-board-mount-pressure-abp-series-datasheet-32305128-ciid-155789.pdf)
//! - [I2C Communication Guidelines](https://sps-support.honeywell.com/s/article/AST-ABP-I2C-Protocol-Guidelines)

#![no_std]

use core::str::FromStr;
use bitmatch::bitmatch;
use embedded_hal::i2c::I2c;

/// Output count at maximum pressure (90 % of 2¹⁴).
const O_MAX: u16 = 0x3999;
/// Output count at minimum pressure (10 % of 2¹⁴).
const O_MIN: u16 = 0x0666;

// ─── Pressure unit ────────────────────────────────────────────────────────────

/// Pressure unit encoded in the Honeywell ABP part number.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PressureUnit {
    /// Bar (B in part number).
    Bar,
    /// Millibar (M in part number).
    Mbar,
    /// Kilopascal (K in part number).
    Kpa,
    /// Pounds per square inch (P in part number).
    Psi,
}

impl PressureUnit {
    /// Returns the factor to multiply a native-unit value by to obtain Pascals.
    ///
    /// `pressure_pa = native_value * unit.to_pa_factor()`
    pub fn to_pa_factor(self) -> f32 {
        match self {
            Self::Bar  => 100_000.0,
            Self::Mbar => 100.0,
            Self::Kpa  => 1_000.0,
            Self::Psi  => 6_894.757_3,
        }
    }
}

/// Formats the unit as a human-readable lowercase string suitable for display.
///
/// | Variant | Output |
/// |---------|--------|
/// | `Bar`   | `"bar"` |
/// | `Mbar`  | `"mbar"` |
/// | `Kpa`   | `"kPa"` |
/// | `Psi`   | `"psi"` |
///
/// # Example
/// ```rust,ignore
/// let unit = PressureUnit::Psi;
/// assert_eq!(format!("{unit}"), "psi");
/// // useful for labelled output: format!("{:.1} {}", native, unit) → "30.0 psi"
/// ```
impl core::fmt::Display for PressureUnit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Bar  => "bar",
            Self::Mbar => "mbar",
            Self::Kpa  => "kPa",
            Self::Psi  => "psi",
        })
    }
}

// ─── Parse error ──────────────────────────────────────────────────────────────

/// Error returned by [`AbpConfig::from_part_number`] when the part number is invalid.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// String is shorter than 15 characters.
    TooShort,
    /// First three characters are not `"ABP"`.
    NotAbpFamily,
    /// Characters at positions 7–9 cannot be parsed as a decimal integer.
    InvalidPressureValue,
    /// Character at position 10 is not a recognised pressure unit (`M`, `B`, `K`, `P`).
    InvalidUnit,
    /// Character at position 11 is not `D` (differential) or `G` (gauge).
    InvalidType,
    /// Character at position 12 is `A` or `S` (analog/SPI — not supported), or unrecognised.
    InvalidAddress,
    /// Character at position 13 is not one of `A`, `D`, `S`, `T`.
    InvalidTransferFunction,
}

// ─── Temperature resolution ───────────────────────────────────────────────────

/// Controls how many I2C bytes are read when calling [`Abp::read_with_temperature`].
///
/// - `Approx`: 3-byte read — 8-bit temperature (~0.8 °C resolution, faster)
/// - `Full`:   4-byte read — 11-bit temperature (~0.1 °C resolution)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TemperatureResolution {
    /// 3-byte transaction, 8-bit temperature (~0.8 °C).
    Approx,
    /// 4-byte transaction, 11-bit temperature (~0.1 °C).
    Full,
}

// ─── Sensor configuration ─────────────────────────────────────────────────────

/// Validated configuration parsed from a Honeywell ABP part number.
///
/// All fields are public — you can also construct this directly for testing
/// or when sensor parameters are known at compile time.
///
/// # Example
/// ```rust,ignore
/// // From part number
/// let config = AbpConfig::from_part_number("ABPDNNN030PG2A3")?;
///
/// // Direct construction
/// let config = AbpConfig {
///     p_max: 30.0,
///     p_min: 0.0,
///     unit: PressureUnit::Psi,
///     i2c_address: 0x28,
///     has_thermometer: false,
/// };
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AbpConfig {
    /// Maximum pressure in the sensor's native unit (from part number).
    pub p_max: f32,
    /// Minimum pressure in the sensor's native unit (0 for gauge, −p_max for differential).
    pub p_min: f32,
    /// Pressure unit as encoded in the part number.
    pub unit: PressureUnit,
    /// 7-bit I2C address.
    pub i2c_address: u8,
    /// `true` if the sensor includes a compensated temperature output.
    pub has_thermometer: bool,
}

impl AbpConfig {
    /// Parse a Honeywell ABP part number string into a validated [`AbpConfig`].
    ///
    /// Returns `Err(`[`ParseError`]`)` instead of panicking on invalid input.
    ///
    /// # Part number layout (0-indexed)
    ///
    /// ```text
    /// A B P D N N N 0 3 0 P G 2 A 3
    /// 0 1 2 3 4 5 6 7 8 9 ↑ ↑ ↑ ↑ ↑
    ///                     │ │ │ │ └─ supply voltage (ignored)
    ///                     │ │ │ └─── transfer function: A/S=no thermometer, D/T=thermometer
    ///                     │ │ └───── output: 0–7=I2C addr 0x08–0x78; A/S=analog/SPI (unsupported)
    ///                     │ └─────── type: D=differential (p_min=−p_max), G=gauge (p_min=0)
    ///                     └───────── unit: M=mbar, B=bar, K=kPa, P=psi
    ///                  [7..10]: numeric pressure max (e.g. "030" → 30)
    ///                  [0..3] : "ABP" family identifier
    /// ```
    ///
    /// # Errors
    ///
    /// | Error | Cause |
    /// |-------|-------|
    /// | [`ParseError::TooShort`] | `part_nr.len() < 15` |
    /// | [`ParseError::NotAbpFamily`] | `part_nr[0..3] != "ABP"` |
    /// | [`ParseError::InvalidPressureValue`] | `part_nr[7..10]` is not a decimal integer |
    /// | [`ParseError::InvalidUnit`] | `part_nr[10]` is not `M`, `B`, `K`, or `P` |
    /// | [`ParseError::InvalidType`] | `part_nr[11]` is not `D` or `G` |
    /// | [`ParseError::InvalidAddress`] | `part_nr[12]` is `A` or `S` (analog/SPI), or unrecognised |
    /// | [`ParseError::InvalidTransferFunction`] | `part_nr[13]` is not `A`, `D`, `S`, or `T` |
    ///
    /// # Example
    /// ```rust,ignore
    /// let config = AbpConfig::from_part_number("ABPDNNN030PG2A3")?;
    /// assert_eq!(config.p_max, 30.0);
    /// assert_eq!(config.unit, PressureUnit::Psi);
    /// assert_eq!(config.i2c_address, 0x28);
    /// ```
    pub fn from_part_number(part_nr: &str) -> Result<Self, ParseError> {
        if part_nr.len() < 15 {
            return Err(ParseError::TooShort);
        }
        if &part_nr[0..3] != "ABP" {
            return Err(ParseError::NotAbpFamily);
        }

        let p_max = f32::from_str(&part_nr[7..10])
            .map_err(|_| ParseError::InvalidPressureValue)?;

        let unit = match &part_nr[10..11] {
            "M" => PressureUnit::Mbar,
            "B" => PressureUnit::Bar,
            "K" => PressureUnit::Kpa,
            "P" => PressureUnit::Psi,
            _   => return Err(ParseError::InvalidUnit),
        };

        let p_min = match &part_nr[11..12] {
            "D" => -p_max,
            "G" => 0.0,
            _   => return Err(ParseError::InvalidType),
        };

        let i2c_address: u8 = match &part_nr[12..13] {
            "A" | "S" => return Err(ParseError::InvalidAddress),
            "0" => 0x08,
            "1" => 0x18,
            "2" => 0x28,
            "3" => 0x38,
            "4" => 0x48,
            "5" => 0x58,
            "6" => 0x68,
            "7" => 0x78,
            _   => return Err(ParseError::InvalidAddress),
        };

        let has_thermometer = match &part_nr[13..14] {
            "A" | "S" => false,
            "D" | "T" => true,
            _         => return Err(ParseError::InvalidTransferFunction),
        };

        Ok(AbpConfig { p_max, p_min, unit, i2c_address, has_thermometer })
    }
}

// ─── Driver error ─────────────────────────────────────────────────────────────

/// Errors returned by [`Abp::read`] and [`Abp::read_with_temperature`].
#[derive(Copy, Clone, Debug)]
pub enum AbpError<E> {
    /// Wraps an I2C bus error from the underlying HAL implementation.
    I2c(E),
    /// Sensor is in command mode; measurement unavailable.
    ErrorCommandMode,
    /// Sensor data is not yet ready (stale); retry the read.
    DataNotReady,
    /// Sensor is reporting a diagnostic/fault condition.
    ErrorDiagnosticState,
}

// ─── Internal status ──────────────────────────────────────────────────────────

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum Status {
    Valid      = 0b00,
    Command    = 0b01,
    Stale      = 0b10,
    Diagnostic = 0b11,
}

impl From<u8> for Status {
    fn from(s: u8) -> Self {
        match s {
            0 => Status::Valid,
            1 => Status::Command,
            2 => Status::Stale,
            3 => Status::Diagnostic,
            _ => panic!("illegal status bits"),
        }
    }
}

// ─── Driver ───────────────────────────────────────────────────────────────────

/// Driver for a Honeywell ABP series I2C pressure sensor.
///
/// Generic over an embedded-hal 1.0 [`I2c`] implementation.
#[derive(Debug)]
pub struct Abp<I2C>
where
    I2C: I2c,
{
    i2c: I2C,
    /// Validated sensor configuration.
    pub config: AbpConfig,
}

impl<I2C> Abp<I2C>
where
    I2C: I2c,
{
    /// Creates a new driver instance from an I2C bus and a validated [`AbpConfig`].
    ///
    /// This constructor is infallible. Use [`AbpConfig::from_part_number`] to obtain
    /// a config from a Honeywell part number string, handling any parse errors there.
    pub fn new(i2c: I2C, config: AbpConfig) -> Self {
        Abp { i2c, config }
    }

    /// Reads a pressure value from the sensor (2-byte I2C transaction).
    ///
    /// Returns `Ok(pressure_pa)` on a valid measurement. This is the fastest read path
    /// and does not retrieve temperature data.
    ///
    /// # Errors
    ///
    /// | Error | Cause |
    /// |-------|-------|
    /// | [`AbpError::I2c`] | Underlying I2C bus error |
    /// | [`AbpError::DataNotReady`] | Sensor returned stale data (status `0b10`); retry |
    /// | [`AbpError::ErrorCommandMode`] | Sensor is in command mode (status `0b01`) |
    /// | [`AbpError::ErrorDiagnosticState`] | Sensor reports a fault (status `0b11`) |
    ///
    /// # Example
    /// ```rust,ignore
    /// let pressure_pa = sensor.read()?;
    /// let native = pressure_pa / sensor.config.unit.to_pa_factor();
    /// println!("{:.1} {}", native, sensor.config.unit); // e.g. "30.0 psi"
    /// ```
    pub fn read(&mut self) -> Result<f32, AbpError<I2C::Error>> {
        let mut buffer: [u8; 2] = [0; 2];
        self.i2c
            .read(self.config.i2c_address, &mut buffer)
            .map_err(AbpError::I2c)?;

        let (raw_status, pressure_counts) = decode_pressure(&buffer);

        match raw_status.into() {
            Status::Valid      => Ok(self.convert_pressure(f32::from(pressure_counts))),
            Status::Command    => Err(AbpError::ErrorCommandMode),
            Status::Stale      => Err(AbpError::DataNotReady),
            Status::Diagnostic => Err(AbpError::ErrorDiagnosticState),
        }
    }

    /// Reads pressure and optionally temperature from the sensor.
    ///
    /// The `resolution` argument controls the I2C transaction size and temperature precision:
    ///
    /// | Resolution | Bytes read | Temperature bits | Precision |
    /// |------------|-----------|-----------------|-----------|
    /// | [`TemperatureResolution::Approx`] | 3 | 8 (T\[10:3\]) | ~0.8 °C |
    /// | [`TemperatureResolution::Full`]   | 4 | 11 (T\[10:0\]) | ~0.1 °C |
    ///
    /// Temperature is always assembled as an 11-bit count before applying the formula;
    /// `Approx` simply leaves the lower 3 bits as zero.
    ///
    /// Returns `(pressure_pa, Some(temp_c))` when `config.has_thermometer` is `true`,
    /// or `(pressure_pa, None)` when the sensor has no thermometer — regardless of
    /// which resolution was requested.
    ///
    /// # Errors
    ///
    /// | Error | Cause |
    /// |-------|-------|
    /// | [`AbpError::I2c`] | Underlying I2C bus error |
    /// | [`AbpError::DataNotReady`] | Sensor returned stale data (status `0b10`); retry |
    /// | [`AbpError::ErrorCommandMode`] | Sensor is in command mode (status `0b01`) |
    /// | [`AbpError::ErrorDiagnosticState`] | Sensor reports a fault (status `0b11`) |
    ///
    /// # Example
    /// ```rust,ignore
    /// let (pressure_pa, temp_opt) =
    ///     sensor.read_with_temperature(TemperatureResolution::Full)?;
    ///
    /// if let Some(temp_c) = temp_opt {
    ///     println!("{:.1} °C", temp_c);
    /// }
    /// ```
    pub fn read_with_temperature(
        &mut self,
        resolution: TemperatureResolution,
    ) -> Result<(f32, Option<f32>), AbpError<I2C::Error>> {
        let (raw_status, pressure_counts, temp_counts) = match resolution {
            TemperatureResolution::Approx => {
                let mut buf: [u8; 3] = [0; 3];
                self.i2c
                    .read(self.config.i2c_address, &mut buf)
                    .map_err(AbpError::I2c)?;
                let (s, p) = decode_pressure(&[buf[0], buf[1]]);
                let t = (buf[2] as u16) << 3;
                (s, p, t)
            }
            TemperatureResolution::Full => {
                let mut buf: [u8; 4] = [0; 4];
                self.i2c
                    .read(self.config.i2c_address, &mut buf)
                    .map_err(AbpError::I2c)?;
                let (s, p) = decode_pressure(&[buf[0], buf[1]]);
                let t = ((buf[2] as u16) << 3) | ((buf[3] as u16) >> 5);
                (s, p, t)
            }
        };

        let pressure_pa = self.convert_pressure(f32::from(pressure_counts));
        let temperature = if self.config.has_thermometer {
            Some(Self::convert_temperature(f32::from(temp_counts)))
        } else {
            None
        };

        match raw_status.into() {
            Status::Valid      => Ok((pressure_pa, temperature)),
            Status::Command    => Err(AbpError::ErrorCommandMode),
            Status::Stale      => Err(AbpError::DataNotReady),
            Status::Diagnostic => Err(AbpError::ErrorDiagnosticState),
        }
    }

    /// Maps a 14-bit output count to pressure in Pascals using the datasheet linear formula.
    ///
    /// ```text
    /// native = (p_max − p_min) / (O_MAX − O_MIN) × (reading − O_MIN) + p_min
    /// result = native × unit.to_pa_factor()
    /// ```
    ///
    /// where `O_MAX = 0x3999` (90 % of 2¹⁴) and `O_MIN = 0x0666` (10 % of 2¹⁴).
    fn convert_pressure(&self, reading: f32) -> f32 {
        let native = (self.config.p_max - self.config.p_min)
            / f32::from(O_MAX - O_MIN)
            * (reading - f32::from(O_MIN))
            + self.config.p_min;
        native * self.config.unit.to_pa_factor()
    }

    /// Converts an assembled 11-bit temperature count to degrees Celsius.
    ///
    /// Formula from the Honeywell datasheet:
    ///
    /// ```text
    /// temp_c = (counts / 2047.0) × 200.0 − 50.0
    /// ```
    ///
    /// Valid range: `counts` in `0..=2047` → `−50.0 °C` to `+150.0 °C`.
    fn convert_temperature(counts: f32) -> f32 {
        (counts / 2047.0) * 200.0 - 50.0
    }
}

// ─── Bit decoding ─────────────────────────────────────────────────────────────

/// Extracts the 2-bit status field and 14-bit pressure count from a raw 2-byte sensor response.
///
/// The Honeywell ABP sensor encodes the response as follows:
///
/// ```text
/// Byte 0:  [ s s p p p p p p ]
/// Byte 1:  [ p p p p p p p p ]
///            ↑↑ └──────────────── 14-bit pressure count (bits 13..0)
///            └┴─────────────────── 2-bit status (bits 7..6 of byte 0)
/// ```
///
/// Returns `(status_bits, pressure_counts)` where `status_bits` is in `0..=3`
/// and maps to [`Status`] via `From<u8>`.
#[bitmatch]
fn decode_pressure(buffer: &[u8; 2]) -> (u8, u16) {
    #[bitmatch]
    let "sspppppp" = buffer[0];
    #[bitmatch]
    let "qqqqqqqq" = buffer[1];

    (bitpack!("000000ss"), bitpack!("00ppppppqqqqqqqq"))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ──

    fn gauge_psi_config() -> AbpConfig {
        AbpConfig {
            p_max: 150.0,
            p_min: 0.0,
            unit: PressureUnit::Psi,
            i2c_address: 0x28,
            has_thermometer: false,
        }
    }

    fn pressure_native(config: &AbpConfig, reading: f32) -> f32 {
        (config.p_max - config.p_min) / f32::from(O_MAX - O_MIN)
            * (reading - f32::from(O_MIN))
            + config.p_min
    }

    // ── PressureUnit ──

    #[test]
    fn pressure_unit_psi_factor() {
        assert!((PressureUnit::Psi.to_pa_factor() - 6_894.757_3).abs() < 1e-3);
    }

    #[test]
    fn pressure_unit_kpa_factor() {
        assert!((PressureUnit::Kpa.to_pa_factor() - 1_000.0).abs() < 1e-6);
    }

    #[test]
    fn pressure_unit_mbar_factor() {
        assert!((PressureUnit::Mbar.to_pa_factor() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn pressure_unit_bar_factor() {
        assert!((PressureUnit::Bar.to_pa_factor() - 100_000.0).abs() < 1e-3);
    }

    // ── AbpConfig::from_part_number ──

    #[test]
    fn from_part_number_valid_gauge_psi() {
        let c = AbpConfig::from_part_number("ABPDNNN030PG2A3").unwrap();
        assert!((c.p_max - 30.0).abs() < 1e-3);
        assert!((c.p_min - 0.0).abs() < 1e-6);
        assert_eq!(c.unit, PressureUnit::Psi);
        assert_eq!(c.i2c_address, 0x28);
        assert!(!c.has_thermometer);
    }

    #[test]
    fn from_part_number_valid_differential_kpa_thermometer() {
        let c = AbpConfig::from_part_number("ABPDNNN100KD3D3").unwrap();
        assert!((c.p_max - 100.0).abs() < 1e-3);
        assert!((c.p_min - (-100.0)).abs() < 1e-3);
        assert_eq!(c.unit, PressureUnit::Kpa);
        assert_eq!(c.i2c_address, 0x38);
        assert!(c.has_thermometer);
    }

    #[test]
    fn from_part_number_too_short() {
        assert_eq!(AbpConfig::from_part_number("ABPDNNN"), Err(ParseError::TooShort));
    }

    #[test]
    fn from_part_number_not_abp() {
        assert_eq!(AbpConfig::from_part_number("XBPDNNN030PG2A3"), Err(ParseError::NotAbpFamily));
    }

    #[test]
    fn from_part_number_invalid_pressure_value() {
        assert_eq!(AbpConfig::from_part_number("ABPDNNNXXXPG2A3"), Err(ParseError::InvalidPressureValue));
    }

    #[test]
    fn from_part_number_invalid_unit() {
        assert_eq!(AbpConfig::from_part_number("ABPDNNN030XG2A3"), Err(ParseError::InvalidUnit));
    }

    #[test]
    fn from_part_number_invalid_type() {
        assert_eq!(AbpConfig::from_part_number("ABPDNNN030PX2A3"), Err(ParseError::InvalidType));
    }

    #[test]
    fn from_part_number_spi_address_rejected() {
        assert_eq!(AbpConfig::from_part_number("ABPDNNN030PGAA3"), Err(ParseError::InvalidAddress));
    }

    #[test]
    fn from_part_number_invalid_transfer_fn() {
        assert_eq!(AbpConfig::from_part_number("ABPDNNN030PG2X3"), Err(ParseError::InvalidTransferFunction));
    }

    // ── Pressure formula ──

    #[test]
    fn convert_pressure_at_o_min_returns_p_min() {
        let cfg = gauge_psi_config();
        let result = pressure_native(&cfg, f32::from(O_MIN));
        assert!(result.abs() < 1e-3, "at o_min expected 0, got {result}");
    }

    #[test]
    fn convert_pressure_at_o_max_returns_p_max() {
        let cfg = gauge_psi_config();
        let result = pressure_native(&cfg, f32::from(O_MAX));
        assert!((result - 150.0).abs() < 1e-3, "at o_max expected 150, got {result}");
    }

    #[test]
    fn convert_pressure_mid_scale() {
        let cfg = gauge_psi_config();
        let mid = f32::from(O_MIN) + f32::from(O_MAX - O_MIN) / 2.0;
        let result = pressure_native(&cfg, mid);
        assert!((result - 75.0).abs() < 0.5, "mid-scale expected ~75, got {result}");
    }

    #[test]
    fn convert_pressure_differential_at_o_min() {
        let cfg = AbpConfig { p_max: 150.0, p_min: -150.0, unit: PressureUnit::Psi,
                              i2c_address: 0x28, has_thermometer: false };
        let result = pressure_native(&cfg, f32::from(O_MIN));
        assert!((result - (-150.0)).abs() < 1e-3, "diff at o_min expected -150, got {result}");
    }

    // ── decode_pressure status bits ──

    #[test]
    fn decode_pressure_valid_status() {
        let buffer: [u8; 2] = [0x12, 0x34];
        let (status, pressure) = decode_pressure(&buffer);
        assert_eq!(status, 0);
        assert_eq!(pressure, 0x1234);
    }

    #[test]
    fn decode_pressure_stale_status() {
        let (status, _) = decode_pressure(&[0b10_000000, 0x00]);
        assert_eq!(status, 2);
    }

    #[test]
    fn decode_pressure_command_status() {
        let (status, _) = decode_pressure(&[0b01_000000, 0x00]);
        assert_eq!(status, 1);
    }

    #[test]
    fn decode_pressure_diagnostic_status() {
        let (status, _) = decode_pressure(&[0b11_000000, 0x00]);
        assert_eq!(status, 3);
    }

    // ── Temperature assembly ──

    #[test]
    fn temperature_approx_low_bits_zero() {
        // Approx: T[10:3] from byte2, low 3 bits = 0
        let byte2: u8 = 0b10110100; // T[10:3] = 0b10110100
        let approx_counts = (byte2 as u16) << 3; // = 0b10110100_000 = 1440
        let temp = (approx_counts as f32 / 2047.0) * 200.0 - 50.0;
        assert!((temp - ((1440.0 / 2047.0) * 200.0 - 50.0)).abs() < 1e-4);
    }

    #[test]
    fn temperature_full_captures_low_bits() {
        let byte2: u8 = 0b10110100;
        let byte3: u8 = 0b101_00000; // T[2:0] = 0b101
        let approx = (byte2 as u16) << 3;
        let full   = ((byte2 as u16) << 3) | ((byte3 as u16) >> 5);
        assert_eq!(approx, 1440);
        assert_eq!(full,   1445); // 1440 | 5
        assert!(full > approx);
        let diff_c = ((full as f32 - approx as f32) / 2047.0) * 200.0;
        assert!(diff_c < 0.8, "precision diff should be < 0.8°C, got {diff_c}");
    }

    #[test]
    fn temperature_approx_full_agree_when_low_bits_zero() {
        let byte2: u8 = 0xA4;
        let byte3: u8 = 0b000_00000; // T[2:0] = 0b000
        let approx = (byte2 as u16) << 3;
        let full   = ((byte2 as u16) << 3) | ((byte3 as u16) >> 5);
        assert_eq!(approx, full);
    }
}
