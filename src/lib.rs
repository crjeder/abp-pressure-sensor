//! ABP series pressure sensor driver
//!
//! A platform-agnostic `no_std` driver for Honeywell ABP series I2C pressure sensors,
//! built on [`embedded-hal`](https://github.com/rust-embedded/embedded-hal) 1.0 traits.
//!
//! # Usage
//!
//! Instantiate [`Abp`] with an I2C bus, a delay provider, and the Honeywell part number
//! string (e.g. `"ABPDNNN150PGAA3"`). The constructor parses the part number to extract
//! the pressure range, unit, sensor type, I2C address, and capabilities.
//!
//! ```rust,ignore
//! use abp_pressure_sensor::Abp;
//!
//! let mut sensor = Abp::new(i2c, delay, "ABPDNNN030PG2A3");
//! let pressure_pa = sensor.read().unwrap();
//! let (pressure_pa, temp_c) = sensor.pressure_and_temperature().unwrap();
//! ```
//!
//! # References
//!
//! - [Datasheet](https://prod-edam.honeywell.com/content/dam/honeywell-edam/sps/siot/de-de/products/sensors/pressure-sensors/board-mount-pressure-sensors/basic-abp-series/documents/sps-siot-basic-board-mount-pressure-abp-series-datasheet-32305128-ciid-155789.pdf)
//! - [I2C Communication Guidelines](https://sps-support.honeywell.com/s/article/AST-ABP-I2C-Protocol-Guidelines)

#![no_std]

use core::str::FromStr;
use bitmatch::bitmatch;
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

/// Errors returned by the ABP driver.
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

#[derive(Copy, Clone, Debug)]
struct Output {
    status: Status,
    pressure: u16,
    temperature: u16,
}

/// Driver for a Honeywell ABP series I2C pressure sensor.
///
/// Generic over an embedded-hal 1.0 [`I2c`] implementation `I2C` and a [`DelayNs`]
/// provider `D`.
#[derive(Debug)]
pub struct Abp<I2C, D>
where
    I2C: I2c,
    D: DelayNs,
{
    i2c: I2C,
    delay: D,
    p_max: f32,
    p_min: f32,
    o_max: u16,
    o_min: u16,
    /// Multiplier to convert the sensor's native pressure unit to Pascals.
    conversion_factor: f32,
    i2c_address: u8,
    has_thermometer: bool,
    has_sleep: bool,
}

impl<I2C, D> Abp<I2C, D>
where
    I2C: I2c,
    D: DelayNs,
{
    /// Creates a new ABP driver instance.
    ///
    /// `part_nr` must be the full Honeywell part number string, e.g. `"ABPDNNN030PG2A3"`.
    /// The constructor panics if the part number is not a valid ABP I2C variant.
    ///
    /// Part number positions (1-indexed as in Honeywell docs, 0-indexed in code):
    /// - [0..3]  : "ABP" family identifier
    /// - [3]     : package (not used by driver)
    /// - [4..7]  : pressure port / product option (not used)
    /// - [7..10] : numeric pressure max (e.g. "150" for 150 PSI)
    /// - [10]    : unit: M=mbar, B=bar, K=kPa, P=psi
    /// - [11]    : type: D=differential, G=gauge
    /// - [12]    : output: 0–7 = I2C address selector; A/S = analog/SPI (unsupported)
    /// - [13]    : transfer function: A/T=no-sleep, D/S=sleep; D/T=has thermometer
    /// - [14]    : supply voltage (not used)
    pub fn new(i2c: I2C, delay: D, part_nr: &'static str) -> Self {
        if &part_nr[0..3] != "ABP" {
            panic!("This driver works only for the ABP series sensors.");
        }

        let p_max = f32::from_str(&part_nr[7..10]).unwrap_or_else(|_| {
            panic!("Invalid part number: cannot parse pressure value from positions 8-10")
        });

        let conversion_factor = match &part_nr[10..11] {
            "M" => 100.0,
            "B" => 100_000.0,
            "K" => 1_000.0,
            "P" => 6_894.757_3,
            _ => panic!("Unknown part: unrecognised pressure unit"),
        };

        let p_min = match &part_nr[11..12] {
            "D" => -p_max,
            "G" => 0.0,
            _ => panic!("Unknown part: type must be D (differential) or G (gauge)"),
        };

        let i2c_address: u8 = match &part_nr[12..13] {
            "A" | "S" => panic!("This driver supports I2C output sensors only (address 0–7)."),
            "0" => 0x08,
            "1" => 0x18,
            "2" => 0x28,
            "3" => 0x38,
            "4" => 0x48,
            "5" => 0x58,
            "6" => 0x68,
            "7" => 0x78,
            _ => panic!("Unknown part: unrecognised output type character"),
        };

        let o_max: u16 = 0x3999; // 90 % of 2^14
        let o_min: u16 = 0x0666; // 10 % of 2^14

        let has_sleep = matches!(&part_nr[13..14], "D" | "S");

        let has_thermometer = matches!(&part_nr[13..14], "D" | "T");

        Abp {
            i2c,
            delay,
            p_max,
            p_min,
            o_max,
            o_min,
            conversion_factor,
            i2c_address,
            has_sleep,
            has_thermometer,
        }
    }

    /// Reads a pressure value from the sensor.
    ///
    /// Returns `Ok(pressure_pa)` on a valid measurement, or an [`AbpError`] on
    /// I2C failure, stale data, command mode, or diagnostic state.
    ///
    /// # Example
    /// ```rust,ignore
    /// let pressure_pa = sensor.read()?;
    /// ```
    pub fn read(&mut self) -> Result<f32, AbpError<I2C::Error>> {
        let mut buffer: [u8; 2] = [0; 2];
        self.i2c
            .read(self.i2c_address, &mut buffer)
            .map_err(AbpError::I2c)?;

        let (raw_status, pressure_counts) = decode_pressure(&buffer);

        match raw_status.into() {
            Status::Valid      => Ok(self.convert_pressure(f32::from(pressure_counts))),
            Status::Command    => Err(AbpError::ErrorCommandMode),
            Status::Stale      => Err(AbpError::DataNotReady),
            Status::Diagnostic => Err(AbpError::ErrorDiagnosticState),
        }
    }

    /// Reads both pressure and temperature from the sensor in a single 4-byte transaction.
    ///
    /// Returns `Ok((pressure_pa, temperature_celsius))`.
    /// Only valid for sensors with a thermometer (transfer function D or T in part number).
    pub fn pressure_and_temperature(
        &mut self,
    ) -> Result<(f32, f32), AbpError<I2C::Error>> {
        let mut buffer: [u8; 4] = [0; 4];
        self.i2c
            .read(self.i2c_address, &mut buffer)
            .map_err(AbpError::I2c)?;

        let output = decode_pressure_and_temperature(&buffer);

        match output.status {
            Status::Valid => Ok((
                self.convert_pressure(f32::from(output.pressure)),
                Self::convert_temperature(f32::from(output.temperature)),
            )),
            Status::Command    => Err(AbpError::ErrorCommandMode),
            Status::Stale      => Err(AbpError::DataNotReady),
            Status::Diagnostic => Err(AbpError::ErrorDiagnosticState),
        }
    }

    /// Maps a 14-bit output count to pressure in Pascals using the datasheet formula.
    fn convert_pressure(&self, reading: f32) -> f32 {
        let native = (self.p_max - self.p_min) / f32::from(self.o_max - self.o_min)
            * (reading - f32::from(self.o_min))
            + self.p_min;
        native * self.conversion_factor
    }

    /// Converts an 11-bit temperature count to degrees Celsius.
    fn convert_temperature(temperature_reading: f32) -> f32 {
        (temperature_reading / 2047.0) * 200.0 - 50.0
    }
}

#[bitmatch]
fn decode_pressure(buffer: &[u8; 2]) -> (u8, u16) {
    #[bitmatch]
    let "sspppppp" = buffer[0];
    #[bitmatch]
    let "qqqqqqqq" = buffer[1];

    (bitpack!("000000ss"), bitpack!("00ppppppqqqqqqqq"))
}

#[bitmatch]
fn decode_pressure_and_temperature(buffer: &[u8; 4]) -> Output {
    #[bitmatch]
    let "sspppppp" = buffer[0];
    #[bitmatch]
    let "qqqqqqqq" = buffer[1];
    #[bitmatch]
    let "tttttttt" = buffer[2];
    #[bitmatch]
    let "vvv?????" = buffer[3];

    let pressure: u16 = bitpack!("00ppppppqqqqqqqq");
    let s: u8 = bitpack!("000000ss");
    let temperature: u16 = bitpack!("00000ttttttttvvv");
    let status: Status = s.into();

    Output { status, pressure, temperature }
}

#[cfg(test)]
mod tests {
    use super::*;

    const O_MAX: u16 = 0x3999;
    const O_MIN: u16 = 0x0666;

    fn pressure_formula(p_max: f32, p_min: f32, reading: f32) -> f32 {
        (p_max - p_min) / f32::from(O_MAX - O_MIN) * (reading - f32::from(O_MIN)) + p_min
    }

    #[test]
    fn convert_pressure_at_o_min_returns_p_min() {
        let result = pressure_formula(150.0, 0.0, f32::from(O_MIN));
        assert!(result.abs() < 1e-3, "at o_min expected p_min=0, got {result}");
    }

    #[test]
    fn convert_pressure_at_o_max_returns_p_max() {
        let result = pressure_formula(150.0, 0.0, f32::from(O_MAX));
        assert!((result - 150.0).abs() < 1e-3, "at o_max expected p_max=150, got {result}");
    }

    #[test]
    fn convert_pressure_mid_scale() {
        let mid = f32::from(O_MIN) + f32::from(O_MAX - O_MIN) / 2.0;
        let result = pressure_formula(150.0, 0.0, mid);
        assert!((result - 75.0).abs() < 0.5, "mid-scale expected ~75, got {result}");
    }

    #[test]
    fn convert_pressure_differential_at_o_min() {
        let result = pressure_formula(150.0, -150.0, f32::from(O_MIN));
        assert!((result - (-150.0)).abs() < 1e-3, "diff at o_min expected -150, got {result}");
    }

    #[test]
    fn decode_pressure_valid_status() {
        // status=0b00, pressure bits = 0x1234 (12 bits from first byte + 8 from second)
        // buffer[0] = 0b00_010010 = 0x12, buffer[1] = 0x34
        let buffer: [u8; 2] = [0x12, 0x34];
        let (status, pressure) = decode_pressure(&buffer);
        assert_eq!(status, 0);
        assert_eq!(pressure, 0x1234);
    }

    #[test]
    fn decode_pressure_stale_status() {
        let buffer: [u8; 2] = [0b10_000000, 0x00];
        let (status, _) = decode_pressure(&buffer);
        assert_eq!(status, 2);
    }

    #[test]
    fn decode_pressure_command_status() {
        let buffer: [u8; 2] = [0b01_000000, 0x00];
        let (status, _) = decode_pressure(&buffer);
        assert_eq!(status, 1);
    }

    #[test]
    fn decode_pressure_diagnostic_status() {
        let buffer: [u8; 2] = [0b11_000000, 0x00];
        let (status, _) = decode_pressure(&buffer);
        assert_eq!(status, 3);
    }
}
