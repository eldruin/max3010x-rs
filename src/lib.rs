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

/// LEDs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Led {
    /// LED1 corresponds to Red in MAX30102
    Led1,
    /// LED1 corresponds to IR in MAX30102
    Led2,
    /// Select all available LEDs in the device
    All,
}

const DEVICE_ADDRESS: u8 = 0b101_0111;

struct Register;

impl Register {
    const FIFO_WR_PTR: u8 = 0x04;
    const FIFO_DATA: u8 = 0x07;
    const FIFO_CONFIG: u8 = 0x08;
    const MODE: u8 = 0x09;
    const LED1_PA: u8 = 0x0C;
    const LED2_PA: u8 = 0x0D;
    const TEMP_INT: u8 = 0x1F;
    const TEMP_CONFIG: u8 = 0x21;
    const REV_ID: u8 = 0xFE;
    const PART_ID: u8 = 0xFF;
}

struct BitFlags;
impl BitFlags {
    const TEMP_EN: u8 = 0b0000_0001;
    const SHUTDOWN: u8 = 0b1000_0000;
    const RESET: u8 = 0b0100_0000;
    const FIFO_ROLLOVER_EN: u8 = 0b0001_0000;
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
    fn with_low(&self, mask: u8) -> Self {
        Config {
            bits: self.bits & !mask,
        }
    }
}

#[doc(hidden)]
pub mod marker {
    pub mod mode {
        pub struct None(());
        pub struct HeartRate(());
    }
    pub mod ic {
        pub struct Max30102(());
    }
}

#[doc(hidden)]
pub trait ChannelCount<IC, MODE>: private::Sealed {
    const CHANNEL_COUNT: u8;
}

impl ChannelCount<marker::ic::Max30102, marker::mode::HeartRate> for marker::mode::HeartRate {
    const CHANNEL_COUNT: u8 = 1;
}

/// MAX3010x device driver.
#[derive(Debug, Default)]
pub struct Max3010x<I2C, IC, MODE> {
    /// The concrete I²C device implementation.
    i2c: I2C,
    temperature_measurement_started: bool,
    mode: Config,
    fifo_config: Config,
    _ic: PhantomData<IC>,
    _mode: PhantomData<MODE>,
}

impl<I2C, E> Max3010x<I2C, marker::ic::Max30102, marker::mode::None>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Create new instance of the MAX3010x device.
    pub fn new_max30102(i2c: I2C) -> Self {
        Max3010x {
            i2c,
            temperature_measurement_started: false,
            mode: Config { bits: 0 },
            fifo_config: Config { bits: 0 },
            _ic: PhantomData,
            _mode: PhantomData,
        }
    }

    /// Change into heart-rate mode.
    ///
    /// This changes the mode and clears the FIFO data.
    pub fn into_heart_rate(
        mut self,
    ) -> Result<Max3010x<I2C, marker::ic::Max30102, marker::mode::HeartRate>, Error<E>> {
        let mode = self.mode.with_low(0b0000_0101).with_high(0b0000_0010);
        self.change_mode(mode)?;
        self.clear_fifo()?;
        let dev = Max3010x {
            i2c: self.i2c,
            temperature_measurement_started: self.temperature_measurement_started,
            mode: self.mode,
            fifo_config: self.fifo_config,
            _ic: PhantomData,
            _mode: PhantomData,
        };
        Ok(dev)
    }
}

impl<I2C, IC, MODE> Max3010x<I2C, IC, MODE> {
    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy(self) -> I2C {
        self.i2c
    }
}

macro_rules! flip_flag_method_impl {
    ($name:ident, $doc:expr, $reg:ident, $reg_variable:ident, $config_method:ident, $bitflag:ident) => {
        #[doc = $doc]
        pub fn $name(&mut self) -> Result<(), Error<E>> {
            let $reg_variable = self.$reg_variable.$config_method(BitFlags::$bitflag);
            self.write_data(&[Register::$reg, $reg_variable.bits])?;
            self.$reg_variable = $reg_variable;
            Ok(())
        }
    }
}

macro_rules! high_low_flag_impl {
    ($enable_name:ident, $enable_doc:expr, $disable_name:ident, $disable_doc:expr, $reg:ident, $reg_variable:ident, $bitflag:ident) => {
        flip_flag_method_impl!(
            $enable_name,
            $enable_doc,
            $reg,
            $reg_variable,
            with_high,
            $bitflag
        );
        flip_flag_method_impl!(
            $disable_name,
            $disable_doc,
            $reg,
            $reg_variable,
            with_low,
            $bitflag
        );
    };
}

impl<I2C, E, IC, MODE> Max3010x<I2C, IC, MODE>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Trigger a software reset
    pub fn reset(&mut self) -> Result<(), Error<E>> {
        let mode = self.mode.with_high(BitFlags::RESET);
        self.write_data(&[Register::MODE, mode.bits])
    }

    /// Put the device in power-save mode.
    pub fn shutdown(&mut self) -> Result<(), Error<E>> {
        let mode = self.mode.with_high(BitFlags::SHUTDOWN);
        self.change_mode(mode)
    }

    /// Wake up from power-save mode.
    pub fn wake_up(&mut self) -> Result<(), Error<E>> {
        let mode = self.mode.with_low(BitFlags::SHUTDOWN);
        self.change_mode(mode)
    }

    /// Set the LED pulse amplitude
    ///
    /// The amplitude value corresponds to a typical current of 0.0 mA for 0
    /// up to 51.0 mA for 255.
    pub fn set_pulse_amplitude(&mut self, led: Led, amplitude: u8) -> Result<(), Error<E>> {
        match led {
            Led::Led1 => self.write_data(&[Register::LED1_PA, amplitude]),
            Led::Led2 => self.write_data(&[Register::LED2_PA, amplitude]),
            Led::All => self.write_data(&[Register::LED1_PA, amplitude, amplitude]),
        }
    }


    /// Resets the FIFO read and write pointers and overflow counter to 0.
    pub fn clear_fifo(&mut self) -> Result<(), Error<E>> {
        self.write_data(&[Register::FIFO_WR_PTR, 0, 0, 0])
    }

    high_low_flag_impl!(
        enable_fifo_rollover,
        "Enable FIFO rollover",
        disable_fifo_rollover,
        "Disable FIFO rollover",
        FIFO_CONFIG,
        fifo_config,
        FIFO_ROLLOVER_EN
    );

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
            self.write_data(&[Register::TEMP_CONFIG, BitFlags::TEMP_EN])
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

    fn change_mode(&mut self, mode: Config) -> Result<(), Error<E>> {
        self.write_data(&[Register::MODE, mode.bits])?;
        self.mode = mode;
        Ok(())
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

    fn write_data(&mut self, data: &[u8]) -> Result<(), Error<E>> {
        self.i2c.write(DEVICE_ADDRESS, data).map_err(Error::I2C)
    }
}

impl<I2C, E, IC, MODE> Max3010x<I2C, IC, MODE>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
    MODE: ChannelCount<IC, MODE>,
{
    /// Reads samples from FIFO.
    ///
    /// Reads data from the FIFO until all the available samples
    /// are read or the input buffer is full.
    ///
    /// Returns the number of _samples_ read.
    ///
    /// The input buffer must contain 3 bytes per channel per sample.
    pub fn read_fifo(&mut self, data: &mut [u8]) -> Result<u8, Error<E>> {
        let mode_channels = usize::from(MODE::CHANNEL_COUNT);
        if data.len() < 3 * mode_channels {
            return Ok(0);
        }
        let samples = self.get_available_sample_count()?;
        let samples_fitting_in_input = data.len() / 3 / mode_channels;
        let sample_count = core::cmp::min(usize::from(samples), samples_fitting_in_input);
        if sample_count != 0 {
            let byte_count = sample_count * mode_channels * 3;
            self.read_data(Register::FIFO_DATA, &mut data[..byte_count])?;
        }
        Ok(sample_count as u8) // the maximum is 32 so this is ok
    }
}

mod private {
    use super::*;
    pub trait Sealed {}

    impl Sealed for marker::mode::HeartRate {}

    impl Sealed for marker::ic::Max30102 {}
}
