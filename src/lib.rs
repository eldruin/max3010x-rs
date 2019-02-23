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
use core::marker::PhantomData;

/// All possible errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus error
    I2C(E),
}

const DEVICE_ADDRESS: u8 = 0b101_0111;

struct Register;

impl Register {
    const FIFO_WR_PTR: u8 = 0x04;
    const MODE: u8 = 0x09;
    const TEMP_INT: u8 = 0x1F;
    const TEMP_CONFIG: u8 = 0x21;
    const REV_ID: u8 = 0xFE;
    const PART_ID: u8 = 0xFF;
}

struct BitFlags;
impl BitFlags {
    const TEMP_EN: u8 = 0b0000_0001;
    const SHUTDOWN: u8 = 0b1000_0000;
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Config {
    bits: u8,
}

impl Config {
    fn with_high(&self, mask: u8) -> Self {
        Config {
            bits: self.bits | mask,
        }
    }
}

#[doc(hidden)]
pub mod marker {
    pub mod mode {
        pub struct None(());
    }
    pub mod ic {
        pub struct Max30102(());
    }
}

/// MAX3010x device driver.
#[derive(Debug, Default)]
pub struct Max3010x<I2C, IC, MODE> {
    /// The concrete I²C device implementation.
    i2c: I2C,
    temperature_measurement_started: bool,
    mode: Config,
    _ic: PhantomData<IC>,
    _mode: PhantomData<MODE>,
}


impl<I2C> Max3010x<I2C, marker::ic::Max30102, marker::mode::None>
{
    /// Create new instance of the MAX3010x device.
    pub fn new_max30102(i2c: I2C) -> Self {
        Max3010x {
            i2c,
            temperature_measurement_started: false,
            mode: Config { bits: 0 },
            _ic: PhantomData,
            _mode: PhantomData,
        }
    }
}

impl<I2C, IC, MODE> Max3010x<I2C, IC, MODE>
{
    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy(self) -> I2C {
        self.i2c
    }
}

impl<I2C, E, IC, MODE> Max3010x<I2C, IC, MODE>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Put the device in power-save mode.
    pub fn shutdown(&mut self) -> Result<(), Error<E>> {
        let mode = self.mode.with_high(BitFlags::SHUTDOWN);
        self.i2c
            .write(DEVICE_ADDRESS, &[Register::MODE, mode.bits])
            .map_err(Error::I2C)?;
        self.mode = mode;
        Ok(())
    }

    /// Perform a temperature measurement.
    ///
    /// This starts a temperature measurement if none is currently ongoing.
    /// When the measurement is finished, returns the result.
    pub fn read_temperature(&mut self) -> nb::Result<f32, Error<E>> {
        let config = self
            .read_register(Register::TEMP_CONFIG)
            .map_err(nb::Error::Other)?;
        if config & BitFlags::TEMP_EN != 0 {
            return Err(nb::Error::WouldBlock);
        }
        if self.temperature_measurement_started {
            let mut data = [0, 0];
            self.read_data(Register::TEMP_INT, &mut data)
                .map_err(nb::Error::Other)?;
            let temp_int = data[0] as i8;
            let temp_frac = f32::from(data[1]) * 0.0625;
            let temp = f32::from(temp_int) + temp_frac;
            self.temperature_measurement_started = false;
            Ok(temp)
        } else {
            self.i2c
                .write(DEVICE_ADDRESS, &[Register::TEMP_CONFIG, BitFlags::TEMP_EN])
                .map_err(Error::I2C)
                .map_err(nb::Error::Other)?;
            self.temperature_measurement_started = true;
            Err(nb::Error::WouldBlock)
        }
    }

    /// Get number of samples available for reading from FIFO
    pub fn get_available_sample_count(&mut self) -> Result<u8, Error<E>> {
        let mut data = [0; 3];
        self.read_data(Register::FIFO_WR_PTR, &mut data)?;
        let wr_ptr = data[0];
        let rd_ptr = data[2];
        let has_rolled_over = rd_ptr > wr_ptr;
        if has_rolled_over {
            Ok(32 - rd_ptr + wr_ptr)
        } else {
            Ok(wr_ptr - rd_ptr)
        }
    }

    /// Get revision ID
    pub fn get_revision_id(&mut self) -> Result<u8, Error<E>> {
        self.read_register(Register::REV_ID)
    }

    /// Get part ID
    pub fn get_part_id(&mut self) -> Result<u8, Error<E>> {
        self.read_register(Register::PART_ID)
    }

    fn read_register(&mut self, register: u8) -> Result<u8, Error<E>> {
        let mut data = [0];
        self.read_data(register, &mut data)?;
        Ok(data[0])
    }

    fn read_data(&mut self, register: u8, data: &mut [u8]) -> Result<(), Error<E>> {
        self.i2c
            .write_read(DEVICE_ADDRESS, &[register], data)
            .map_err(Error::I2C)
    }
}
