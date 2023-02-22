//! ABP series pressure sensor driver
//!
//! This is a platform agnostic driver to interface with the ABP sensors.
//! This driver [no_std] is built using [`embedded-hal`][2] traits.
//!
//!
//! # Usage
//!
//! # Examples
//! ```rust
//! // embedded_hal implementation
//! use rppal::{spi::{Spi, Bus, SlaveSelect, Mode, Error},hal::Delay};
//!
//!
//! // minimal example
//! fn main() -> Result<(), Error>
//! {
//!     Ok(())
//! }
//! ```
//!
//! # References
//!
//! - [Datasheet][1]
//!
//! [1]: https://prod-edam.honeywell.com/content/dam/honeywell-edam/sps/siot/de-de/products/sensors/pressure-sensors/board-mount-pressure-sensors/basic-abp-series/documents/sps-siot-basic-board-mount-pressure-abp-series-datasheet-32305128-ciid-155789.pdf
//!
//! - [`embedded-hal`][2]
//!
//! [2]: https://github.com/rust-embedded/embedded-hal
//! 
//! - [I2C Communication][3]
//!
//! [3]: https://sps-support.honeywell.com/s/article/AST-ABP-I2C-Protocol-Guidelines
//!

#![no_std]

use embedded_hal as hal;
use hal::blocking::{i2c, delay::DelayMs};
use core::str::FromStr;
// use core::error::Error;
use substring::Substring;
use nb::{Error::{Other, WouldBlock}};

// use bitmach to decode the result
use bitmatch::bitmatch;

type I2cError = embedded_hal::blocking::i2c::Read::Error;

#[derive(Copy, Clone, Debug)]
pub enum ApbError<E>
{
    Other(E),
    ErrorCommandMode,
    ErrorDiagnosticState,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum Status
{
    Valid       = 0b00000000,
    Command     = 0b00000001,
    Stale       = 0b00000010,
    Diagnostic  = 0b00000011,
}

impl From<u8> for Status
{
    fn from(s: u8) -> Self
    {
        match s
        {
            0 => Status::Valid,
            1 => Status::Command,
            2 => Status::Stale,
            3 => Status::Diagnostic,
            _ => panic!("illegal staus")
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PressureUnit
{
    Bar,        // bar
    Mbar,       // mbar
    Kpa,        // kPa
    Psi         // psi
}

#[derive(Copy, Clone, Debug)]
struct Output
{
    status: Status,
    pressure: u16,
    temperature: u16
}

/// Represents an instance of a Abp device
#[derive(Debug)]
pub struct Abp<I2C, D>
where
    I2C: i2c::Read,
    D: DelayMs<u16>,
{
    // SPI specific
    i2c: I2C,
    // timeer for delay
    delay: D,
    p_max: f32,
    p_min: f32,
    o_max: u16,
    o_min: u16,
    conversion_factor: f32,
    i2c_address: u8,
    has_thermometer: bool,
    has_sleep: bool
}

impl <I2C, D, E> Abp<I2C, D>
where
    I2C: i2c::Read,
    D: DelayMs<u16>,
{
    // type _embedded_hal_blocking_i2c_error = i2c::Read::Error;
    /// opens a connection to a ABP on a specified I2C.
    ///
    pub fn new(i2c: I2C, delay: D, part_nr: & 'static str) -> Self
    {
        // example part number (without spaces):
        // ABP D NN N 150PG A A 3
        // 000 0 00 0 00111 1 1 1
        // 123 4 56 7 89012 3 4 5

        //let (part, _) = part_nr.split_at(3);
        // Product series
        if part_nr.substring(1, 3) != "ABP"
        {
            panic!("This driver works only for the ABP series sensors.")
        };

        // Package, pressure port and product option [4..7] are not relevant for the driver
        // Pressure range
        let p_max = f32::from_str(part_nr.substring(8, 10)).unwrap();

        // conversion to Pa
        let conversion_factor = match part_nr.substring(11, 11)
        {
            "M" => 100.0,       //mbar
            "B" => 100000.0,    //bar
            "K" => 1000.0,      //kPa 
            "P" => 6894.757293, //psi
             _  => panic!("Unkonwn part: unkonwn pressure unit")
        };

        let p_min = match part_nr.substring(12, 12)
        {
            "D" => -p_max,      // differential type
            "G" => 0.0,         // gauge type
             _  => panic!("Unkown part: Type must be differential or gauge.")
        };

        let i2c_address = match part_nr.substring(13, 13)
        {
            "A" => panic!("This driver is only for the sensors with I2C interface."),
            "S" => panic!("This driver is only for the sensors with I2C interface."),
            "0" => 0x08,
            "1" => 0x18,
            "2" => 0x28,
            "3" => 0x38,
            "4" => 0x48,
            "5" => 0x58,
            "6" => 0x68,
            "7" => 0x78,
             _  => panic!("Unkonw part. Output type {} not known.", part_nr.substring(13, 13))
        };

        let o_max = 0x3999;     // 90 % of 2^14
        let o_min = 0x0666;     // 10 % of 2^14

        let has_sleep = match part_nr.substring(14, 14)
        {
            "A" => false,
            "D" => true,
            "S" => true,
            "T" => false,
             _  => panic!("Unkown part: Transfer function has to be one of A, D, S, or T")
        };

        let has_thermometer = match part_nr.substring(14, 14)
        {
            "A" => false,
            "D" => true,
            "S" => false,
            "T" => true,
             _  => panic!("Unkown part: Transfer function has to be one of A, D, S, or T")
        };
        // Supply voltage [15] is not relevant for the driver

        Abp {i2c, delay, p_max, p_min, o_max, o_min, conversion_factor, i2c_address, has_sleep, has_thermometer}
    }

    /// reads a pressure value from the ADP and retrurns it
    /// # Examples
    /// ```rust
    /// let v = block!(pressure.read())?;
    /// ```
    /// # Errors
    /// Returns i2c errors and nb::Error::WouldBlock if data isn't ready to be read from ADP
    pub fn read(&mut self) -> nb::Result<f32, nb::Error<ApbError<I2cError>>>
    {
        let mut buffer: [u8; 2] = [0; 2];
        self.i2c.read(self.i2c_address, &mut buffer)?;

        let (status, pressure) = decode_pressure(& buffer);

        match status
        {
            Valid => Ok(self.convert_pressure(pressure.into())),
            Command => Err(Other(ApbError::ErrorCommandMode)),
            Stale => Err(nb::Error::WouldBlock),
            Diagnostic => Err(Other(ApbError::ErrorDiagnosticState)),    
        }
    }

    pub fn pressure_and_temperature(&mut self) -> Result<f32, E>
    {
        //if self.has_thermometer == false {return self::Error}
        let mut buffer: [u8; 4] = [0; 4];
        self.i2c.read(self.i2c_address, &mut buffer)?;

        let output: Output = decode_pressure_and_temperature(& buffer);

        Ok(self.convert_pressure(output.pressure.into()))
    }

    fn convert_pressure(& self, reading: f32) -> f32
    {
        (f32::from(self.o_max - self.o_min)/(self.p_max - self.p_min))*(reading - self.p_min) + f32::from(self.o_min)
    }

    fn convert_temperature(& self, temperature_reading: f32) -> f32
    {
        ((temperature_reading/2047.0) * 200.0) - 50.0
    }
}

#[bitmatch]
fn decode_pressure(buffer: &[u8;2]) -> (u8, u16)
{
	#[bitmatch]
	let "ss pp pp pp" = buffer[0];
	#[bitmatch]
	let "qq qq qq qq" = buffer[1];

	//let pressure: u16 = bitpack!("00ppppppqqqqqqqq");
    //let status: u8 = bitpack!("0000000ss");
    (bitpack!("00 00 00 ss"), bitpack!("00ppppppqqqqqqqq"))
}

#[bitmatch]
fn decode_pressure_and_temperature(buffer: &[u8;4]) -> Output
{
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

    Output{status, pressure, temperature}
}
