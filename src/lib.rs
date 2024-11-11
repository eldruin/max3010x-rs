//! This is a platform agnostic Rust driver for the MAX3010x high-sensitivity
//! pulse oximeter and heart-rate sensor for wearable health, based on the
//! [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! This driver allows you to:
//! - Get the number of samples available on the FIFO. See [`get_available_sample_count()`].
//! - Get the number of samples lost from the FIFO. See [`get_overflow_sample_count()`].
//! - Read samples from the FIFO. See [`read_fifo()`].
//! - Perform a temperature measurement. See [`read_temperature()`].
//! - Change into heart-rate, oximeter or multi-LED modes. See [`into_multi_led()`].
//! - Set the sample averaging. See [`set_sample_averaging()`].
//! - Set the LED pulse amplitude. See [`set_pulse_amplitude()`].
//! - Set the LED pulse width. See [`set_pulse_width()`].
//! - Set the sampling rate. See [`set_sampling_rate()`].
//! - Set the ADC range. See [`set_adc_range()`].
//! - Set the LED time slots in multi-LED mode. [`set_led_time_slots()`].
//! - Enable/disable the FIFO rollover. See [`enable_fifo_rollover()`].
//! - Clear the FIFO. See [`clear_fifo()`].
//! - Wake-up and shutdown the device. See [`shutdown()`].
//! - Perform a software reset. See [`reset()`].
//! - Get the device part and revision id. See [`get_part_id()`].
//! - Interrupts:
//!   - Read the status of all interrupts. See [`read_interrupt_status()`].
//!   - Set FIFO-almost-full level interrupt. See [`set_fifo_almost_full_level_interrupt()`].
//!   - Enable/disable the FIFO-almost-full interrupt. See [`enable_fifo_almost_full_interrupt()`].
//!   - Enable/disable the ambient-light-cancellation overflow interrupt. See [`enable_alc_overflow_interrupt()`].
//!   - Enable/disable the temperature-ready interrupt. See [`enable_temperature_ready_interrupt()`].
//!   - Enable/disable the new-FIFO-data-ready interrupt. See [`enable_new_fifo_data_ready_interrupt()`].
//!
//! [`get_available_sample_count()`]: struct.Max3010x.html#method.get_available_sample_count
//! [`get_overflow_sample_count()`]: struct.Max3010x.html#method.get_overflow_sample_count
//! [`read_fifo()`]: struct.Max3010x.html#method.read_fifo
//! [`read_temperature()`]: struct.Max3010x.html#method.read_temperature
//! [`into_multi_led()`]: struct.Max3010x.html#method.into_multi_led
//! [`set_sample_averaging()`]: struct.Max3010x.html#method.set_sample_averaging
//! [`set_pulse_width()`]: struct.Max3010x.html#method.set_pulse_width
//! [`set_pulse_amplitude()`]: struct.Max3010x.html#method.set_pulse_amplitude
//! [`set_sampling_rate()`]: struct.Max3010x.html#method.set_sampling_rate
//! [`set_adc_range()`]: struct.Max3010x.html#method.set_adc_range
//! [`set_led_time_slots()`]: struct.Max3010x.html#method.set_led_time_slots
//! [`shutdown()`]: struct.Max3010x.html#method.shutdown
//! [`reset()`]: struct.Max3010x.html#method.reset
//! [`set_fifo_almost_full_level_interrupt()`]: struct.Max3010x.html#method.set_fifo_almost_full_level_interrupt
//! [`enable_fifo_rollover()`]: struct.Max3010x.html#method.enable_fifo_rollover
//! [`clear_fifo()`]: struct.Max3010x.html#method.clear_fifo
//! [`read_interrupt_status()`]: struct.Max3010x.html#method.read_interrupt_status
//! [`enable_fifo_almost_full_interrupt()`]: struct.Max3010x.html#method.enable_fifo_almost_full_interrupt
//! [`enable_alc_overflow_interrupt()`]: struct.Max3010x.html#method.enable_alc_overflow_interrupt
//! [`enable_temperature_ready_interrupt()`]: struct.Max3010x.html#method.enable_temperature_ready_interrupt
//! [`enable_new_fifo_data_ready_interrupt()`]: struct.Max3010x.html#method.enable_new_fifo_data_ready_interrupt
//! [`get_part_id()`]: struct.Max3010x.html#method.get_part_id
//!
//! ## The device
//! The `MAX30102` is an integrated pulse oximetry and heart-rate monitor module.
//! It includes internal LEDs, photodetectors, optical elements, and low-noise
//! electronics with ambient light rejection. The `MAX30102` provides a complete
//! system solution to ease the design-in process for mobile and
//! wearable devices.
//!
//! The `MAX30102` operates on a single 1.8V power supply and a separate 3.3V
//! power supply for the internal LEDs. Communication is through a standard
//! I2C-compatible interface. The module can be shut down through software
//! with zero standby current, allowing the power rails to remain
//! powered at all times.
//!
//! Datasheet:
//! - [`MAX30102`](https://datasheets.maximintegrated.com/en/ds/MAX30102.pdf)
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
//! ### Read samples in heart-rate mode
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! extern crate max3010x;
//! use max3010x::{Max3010x, Led, SampleAveraging};
//!
//! # fn main() {
//! let dev = hal::I2cdev::new("/dev/i2c-1").unwrap();
//! let mut sensor = Max3010x::new_max30102(dev);
//! let mut sensor = sensor.into_heart_rate().unwrap();
//! sensor.set_sample_averaging(SampleAveraging::Sa4).unwrap();
//! sensor.set_pulse_amplitude(Led::All, 15).unwrap();
//! sensor.enable_fifo_rollover().unwrap();
//! let mut data = [0; 3];
//! let samples_read = sensor.read_fifo(&mut data).unwrap();
//!
//! // get the I2C device back
//! let dev = sensor.destroy();
//! # }
//! ```
//!
//! ### Set led slots in multi-led mode
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! extern crate max3010x;
//! use max3010x::{ Max3010x, Led, TimeSlot };
//!
//! # fn main() {
//! let dev = hal::I2cdev::new("/dev/i2c-1").unwrap();
//! let mut max30102 = Max3010x::new_max30102(dev);
//! let mut max30102 = max30102.into_multi_led().unwrap();
//! max30102.set_pulse_amplitude(Led::All, 15).unwrap();
//! max30102.set_led_time_slots([
//!     TimeSlot::Led1,
//!     TimeSlot::Led2,
//!     TimeSlot::Led1,
//!     TimeSlot::Disabled
//! ]).unwrap();
//! max30102.enable_fifo_rollover().unwrap();
//! let mut data = [0; 2];
//! let samples_read = max30102.read_fifo(&mut data).unwrap();
//!
//! // get the I2C device back
//! let dev = max30102.destroy();
//! # }
//! ```
//!

#![deny(missing_docs, unsafe_code)]
#![no_std]

extern crate embedded_hal as hal;
use hal::i2c;
extern crate nb;
use core::marker::PhantomData;

/// All possible errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus error
    I2C(E),
    /// Invalid arguments provided
    InvalidArguments,
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

/// Multi-LED mode sample time slot configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeSlot {
    /// Time slot is disabled
    Disabled,
    /// LED 1 active during time slot (corresponds to Red in MAX30102)
    Led1,
    /// LED 2 active during time slot (corresponds to IR in MAX30102)
    Led2,
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

/// LED pulse width (determines ADC resolution)
///
/// This is limited by the current mode and the selected sample rate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LedPulseWidth {
    /// 69 μs pulse width (15-bit ADC resolution)
    Pw69,
    /// 118 μs pulse width (16-bit ADC resolution)
    Pw118,
    /// 215 μs pulse width (17-bit ADC resolution)
    Pw215,
    /// 411 μs pulse width (18-bit ADC resolution)
    Pw411,
}

/// Sampling rate
///
/// This is limited by the current mode and the selected LED pulse width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SamplingRate {
    /// 50 samples per second
    Sps50,
    /// 100 samples per second
    Sps100,
    /// 200 samples per second
    Sps200,
    /// 400 samples per second
    Sps400,
    /// 800 samples per second
    Sps800,
    /// 1000 samples per second
    Sps1000,
    /// 1600 samples per second
    Sps1600,
    /// 3200 samples per second
    Sps3200,
}

/// ADC range
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdcRange {
    /// Full scale 2048 nA
    Fs2k,
    /// Full scale 4094 nA
    Fs4k,
    /// Full scale 8192 nA
    Fs8k,
    /// Full scale 16394 nA
    Fs16k,
}

/// Interrupt status flags
#[derive(Debug, Clone, Copy)]
pub struct InterruptStatus {
    /// Power ready interrupt
    pub power_ready: bool,
    /// FIFO almost full interrupt
    pub fifo_almost_full: bool,
    /// New FIFO data ready interrupt
    pub new_fifo_data_ready: bool,
    /// Ambient light cancellation overflow interrupt
    pub alc_overflow: bool,
    /// Internal die temperature conversion ready interrupt
    pub temperature_ready: bool,
}

const DEVICE_ADDRESS: u8 = 0b101_0111;

struct Register;

impl Register {
    const INT_STATUS: u8 = 0x0;
    const INT_EN1: u8 = 0x02;
    const INT_EN2: u8 = 0x03;
    const FIFO_WR_PTR: u8 = 0x04;
    const OVF_COUNTER: u8 = 0x05;
    const FIFO_DATA: u8 = 0x07;
    const FIFO_CONFIG: u8 = 0x08;
    const MODE: u8 = 0x09;
    const SPO2_CONFIG: u8 = 0x0A;
    const LED1_PA: u8 = 0x0C;
    const LED2_PA: u8 = 0x0D;
    const SLOT_CONFIG0: u8 = 0x11;
    const TEMP_INT: u8 = 0x1F;
    const TEMP_CONFIG: u8 = 0x21;
    const REV_ID: u8 = 0xFE;
    const PART_ID: u8 = 0xFF;
}

struct BitFlags;
impl BitFlags {
    const FIFO_A_FULL_INT: u8 = 0b1000_0000;
    const ALC_OVF_INT: u8 = 0b0010_0000;
    const DIE_TEMP_RDY_INT: u8 = 0b0000_0010;
    const PPG_RDY_INT: u8 = 0b0100_0000;
    const PWR_RDY_INT: u8 = 0b0000_0001;
    const TEMP_EN: u8 = 0b0000_0001;
    const SHUTDOWN: u8 = 0b1000_0000;
    const RESET: u8 = 0b0100_0000;
    const FIFO_ROLLOVER_EN: u8 = 0b0001_0000;
    const ADC_RGE0: u8 = 0b0010_0000;
    const ADC_RGE1: u8 = 0b0100_0000;
    const LED_PW0: u8 = 0b0000_0001;
    const LED_PW1: u8 = 0b0000_0010;
    const SPO2_SR0: u8 = 0b0000_0100;
    const SPO2_SR1: u8 = 0b0000_1000;
    const SPO2_SR2: u8 = 0b0001_0000;
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
        pub struct Oximeter(());
        pub struct MultiLed(());
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
    fifo_config: Config,
    spo2_config: Config,
    int_en1: Config,
    int_en2: Config,
    _ic: PhantomData<IC>,
    _mode: PhantomData<MODE>,
}

impl<I2C, E> Max3010x<I2C, marker::ic::Max30102, marker::mode::None>
where
    I2C: i2c::I2c<Error = E>,
{
    /// Create new instance of the MAX3010x device.
    pub fn new_max30102(i2c: I2C) -> Self {
        Max3010x {
            i2c,
            temperature_measurement_started: false,
            mode: Config { bits: 0 },
            fifo_config: Config { bits: 0 },
            spo2_config: Config { bits: 0 },
            int_en1: Config { bits: 0 },
            int_en2: Config { bits: 0 },
            _ic: PhantomData,
            _mode: PhantomData,
        }
    }
}

impl<I2C, E, IC, MODE> Max3010x<I2C, IC, MODE>
where
    I2C: i2c::I2c<Error = E>,
{
    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy(self) -> I2C {
        self.i2c
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), Error<E>> {
        self.i2c.write(DEVICE_ADDRESS, data).map_err(Error::I2C)
    }
}

mod config;
mod reading;

mod private {
    use super::*;
    pub trait Sealed {}

    impl Sealed for marker::mode::HeartRate {}
    impl Sealed for marker::mode::Oximeter {}
    impl Sealed for marker::mode::MultiLed {}

    impl Sealed for marker::ic::Max30102 {}
}
