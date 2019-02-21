//! This is a platform agnostic Rust driver for the MAX3010x high-sensitivity
//! pulse oximeter and heart-rate sensor for wearable health, based on the
//! [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! This driver allows you to:
//! - TODO
//!
//! ## The device
//! TODO
//!
//! ## Usage examples (see also examples folder)
//!
//! To use this driver, import this crate and an `embedded_hal` implementation,
//! then instantiate the device.
//!
//! Please find additional examples using hardware in this repository: [driver-examples]
//!
//! [driver-examples]: https://github.com/eldruin/driver-examples
//!

#![deny(missing_docs, unsafe_code)]
// TODO #![deny(warnings)]
#![no_std]

extern crate embedded_hal as hal;
use hal::blocking::i2c;
extern crate nb;

/// All possible errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus error
    I2C(E),
}

const DEVICE_ADDRESS: u8 = 0b1010111;

struct Register;

impl Register {
    const FIFO_WR_PTR: u8 = 0x04;
    const TEMP_INT: u8 = 0x1F;
    const TEMP_CONFIG: u8 = 0x21;
    const REV_ID: u8 = 0xFE;
    const PART_ID: u8 = 0xFF;
}

struct BitFlags;
impl BitFlags {
    const TEMP_EN: u8 = 0x01;
}

/// MAX3010x device driver.
#[derive(Debug, Default)]
pub struct Max3010x<I2C> {
    /// The concrete I²C device implementation.
    i2c: I2C,
    temperature_measurement_started: bool,
}

impl<I2C, E> Max3010x<I2C>
where
    I2C: i2c::Write<Error = E>,
{
    /// Create new instance of the MAX3010x device.
    pub fn new(i2c: I2C) -> Self {
        Max3010x {
            i2c,
            temperature_measurement_started: false,
        }
    }

    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy(self) -> I2C {
        self.i2c
    }

}

impl<I2C, E> Max3010x<I2C>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Perform a temperature measurement.
    ///
    /// This starts a temperature measurement if none is currently ongoing.
    /// When the measurement is finished, returns the result.
    pub fn read_temperature(&mut self) -> nb::Result<f32, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[Register::TEMP_CONFIG], &mut data)
            .map_err(Error::I2C).map_err(nb::Error::Other)?;
        if data[0] & BitFlags::TEMP_EN != 0 {
            return Err(nb::Error::WouldBlock)
        }
        else {
            if self.temperature_measurement_started {
                let mut data = [0, 0];
                self.i2c
                    .write_read(DEVICE_ADDRESS, &[Register::TEMP_INT], &mut data)
                    .map_err(Error::I2C).map_err(nb::Error::Other)?;
                let temp_int = data[0] as i8;
                let temp_frac = f32::from(data[1]) * 0.0625;
                let temp = f32::from(temp_int) + temp_frac;
                self.temperature_measurement_started = false;
                return Ok(temp)
            }
            else {
                self.i2c
                    .write(DEVICE_ADDRESS, &[Register::TEMP_CONFIG, BitFlags::TEMP_EN])
                    .map_err(Error::I2C).map_err(nb::Error::Other)?;
                self.temperature_measurement_started = true;
                return Err(nb::Error::WouldBlock)
            }
        }
    }

    /// Get number of samples available for reading from FIFO
    pub fn get_available_sample_count(&mut self) -> Result<u8, Error<E>> {
        let mut data = [0; 3];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[Register::FIFO_WR_PTR], &mut data)
            .map_err(Error::I2C)?;
        let wr_ptr = data[0];
        let rd_ptr = data[2];
        let has_rolled_over = rd_ptr > wr_ptr;
        if has_rolled_over {
            Ok(32 - rd_ptr + wr_ptr)
        }
        else {
            Ok(wr_ptr - rd_ptr)
        }
    }

    /// Get revision ID
    pub fn get_revision_id(&mut self) -> Result<u8, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[Register::REV_ID], &mut data)
            .map_err(Error::I2C)?;
        Ok(data[0])
    }

    /// Get part ID
    pub fn get_part_id(&mut self) -> Result<u8, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[Register::PART_ID], &mut data)
            .map_err(Error::I2C)?;
        Ok(data[0])
    }
}