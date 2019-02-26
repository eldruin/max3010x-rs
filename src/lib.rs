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

/// Sample averaging
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SampleAveraging {
    /// 1 (no averaging) (default)
    Sa1,
    /// 2
    Sa2,
    /// 4
    Sa4,
    /// 8
    Sa8,
    /// 16
    Sa16,
    /// 32
    Sa32,
}

/// Number of empty data samples when the FIFO almost full interrupt is issued.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FifoAlmostFullLevelInterrupt {
    /// Interrupt issue when 0 spaces are left in FIFO. (default)
    L0,
    /// Interrupt issue when 1 space is left in FIFO.
    L1,
    /// Interrupt issue when 2 spaces are left in FIFO.
    L2,
    /// Interrupt issue when 3 spaces are left in FIFO.
    L3,
    /// Interrupt issue when 4 spaces are left in FIFO.
    L4,
    /// Interrupt issue when 5 spaces are left in FIFO.
    L5,
    /// Interrupt issue when 6 spaces are left in FIFO.
    L6,
    /// Interrupt issue when 7 spaces are left in FIFO.
    L7,
    /// Interrupt issue when 8 spaces are left in FIFO.
    L8,
    /// Interrupt issue when 9 spaces are left in FIFO.
    L9,
    /// Interrupt issue when 10 spaces are left in FIFO.
    L10,
    /// Interrupt issue when 11 spaces are left in FIFO.
    L11,
    /// Interrupt issue when 12 spaces are left in FIFO.
    L12,
    /// Interrupt issue when 13 spaces are left in FIFO.
    L13,
    /// Interrupt issue when 14 spaces are left in FIFO.
    L14,
    /// Interrupt issue when 15 spaces are left in FIFO.
    L15,
}

impl FifoAlmostFullLevelInterrupt {
    fn get_register_value(self) -> u8 {
        use FifoAlmostFullLevelInterrupt as L;
        match self {
            L::L0 => 0,
            L::L1 => 1,
            L::L2 => 2,
            L::L3 => 3,
            L::L4 => 4,
            L::L5 => 5,
            L::L6 => 6,
            L::L7 => 7,
            L::L8 => 8,
            L::L9 => 9,
            L::L10 => 10,
            L::L11 => 11,
            L::L12 => 12,
            L::L13 => 13,
            L::L14 => 14,
            L::L15 => 15,
        }
    }
}

/// Interrupt status flags
#[derive(Debug, Clone)]
pub struct InterruptStatus {
    /// Power ready interrupt
    pub power_ready: bool,
    /// FIFO almost full interrupt
    pub fifo_almost_full: bool,
}

const DEVICE_ADDRESS: u8 = 0b101_0111;

struct Register;

impl Register {
    const INT_STATUS: u8 = 0x0;
    const INT_EN1: u8 = 0x02;
    const INT_EN2: u8 = 0x03;
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
    const FIFO_A_FULL_INT_EN: u8 = 0b1000_0000;
    const ALC_OVF_INT_EN: u8 = 0b0010_0000;
    const DIE_TEMP_RDY_INT_EN: u8 = 0b0000_0010;
    const PWR_RDY: u8 = 0b0000_0001;
    const FIFO_A_FULL: u8 = 0b1000_0000;
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
    int_en1: Config,
    int_en2: Config,
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
            int_en1: Config { bits: 0 },
            int_en2: Config { bits: 0 },
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
            int_en1: self.int_en1,
            int_en2: self.int_en2,
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

    /// Set sample averaging
    pub fn set_sample_averaging(
        &mut self,
        sample_averaging: SampleAveraging,
    ) -> Result<(), Error<E>> {
        let fifo_config = self.fifo_config.with_low(0b1110_0000);
        let fifo_config = match sample_averaging {
            SampleAveraging::Sa1 => fifo_config,
            SampleAveraging::Sa2 => fifo_config.with_high(0b0010_0000),
            SampleAveraging::Sa4 => fifo_config.with_high(0b0100_0000),
            SampleAveraging::Sa8 => fifo_config.with_high(0b0110_0000),
            SampleAveraging::Sa16 => fifo_config.with_high(0b1000_0000),
            SampleAveraging::Sa32 => fifo_config.with_high(0b1010_0000),
        };
        self.write_data(&[Register::FIFO_CONFIG, fifo_config.bits])?;
        self.fifo_config = fifo_config;
        Ok(())
    }

    /// Set number of empty data samples available in the FIFO
    /// when a FIFO-almost-full interrupt will be issued.
    pub fn set_fifo_almost_full_level_interrupt(
        &mut self,
        level: FifoAlmostFullLevelInterrupt,
    ) -> Result<(), Error<E>> {
        let fifo_config = self
            .fifo_config
            .with_low(0b0000_0111)
            .with_high(level.get_register_value());
        self.write_data(&[Register::FIFO_CONFIG, fifo_config.bits])?;
        self.fifo_config = fifo_config;
        Ok(())
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

    /// Read status of all interrupts
    pub fn read_interrupt_status(&mut self) -> Result<InterruptStatus, Error<E>> {
        let mut data = [0];
        self.read_data(Register::INT_STATUS, &mut data)?;
        let status = InterruptStatus {
            power_ready: (data[0] & BitFlags::PWR_RDY) != 0,
            fifo_almost_full: (data[0] & BitFlags::FIFO_A_FULL) != 0,
        };
        Ok(status)
    }

    high_low_flag_impl!(
        enable_fifo_almost_full_interrupt,
        "Enable FIFO almost full interrupt",
        disable_fifo_almost_full_interrupt,
        "Disable FIFO almost full interrupt",
        INT_EN1,
        int_en1,
        FIFO_A_FULL_INT_EN
    );

    high_low_flag_impl!(
        enable_alc_overflow_interrupt,
        "Enable ambient light cancellation overflow interrupt",
        disable_alc_overflow_interrupt,
        "Disable ambient light cancellation overflow interrupt",
        INT_EN1,
        int_en1,
        ALC_OVF_INT_EN
    );

    high_low_flag_impl!(
        enable_temperature_ready_interrupt,
        "Enable internal die temperature conversion ready interrupt",
        disable_temperature_ready_interrupt,
        "Disable internal die temperature conversion ready interrupt",
        INT_EN2,
        int_en2,
        DIE_TEMP_RDY_INT_EN
    );

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
