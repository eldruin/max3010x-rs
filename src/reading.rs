//! Reading data method implementation.

use super::{
    marker, private, BitFlags, Error, InterruptStatus, Max3010x, Register, DEVICE_ADDRESS,
};
use hal::blocking::i2c;

#[doc(hidden)]
pub trait ChannelCount<IC, MODE>: private::Sealed {
    const CHANNEL_COUNT: u8;
}

impl ChannelCount<marker::ic::Max30102, marker::mode::HeartRate> for marker::mode::HeartRate {
    const CHANNEL_COUNT: u8 = 1;
}

impl ChannelCount<marker::ic::Max30102, marker::mode::Oximeter> for marker::mode::Oximeter {
    const CHANNEL_COUNT: u8 = 2;
}

impl ChannelCount<marker::ic::Max30102, marker::mode::MultiLED> for marker::mode::MultiLED {
    const CHANNEL_COUNT: u8 = 2;
}

impl<I2C, E, IC, MODE> Max3010x<I2C, IC, MODE>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
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
        let mut data = [0; 2];
        self.read_data(Register::INT_STATUS, &mut data)?;
        let status = InterruptStatus {
            power_ready: (data[0] & BitFlags::PWR_RDY_INT) != 0,
            fifo_almost_full: (data[0] & BitFlags::FIFO_A_FULL_INT) != 0,
            new_fifo_data_ready: (data[0] & BitFlags::PPG_RDY_INT) != 0,
            alc_overflow: (data[0] & BitFlags::ALC_OVF_INT) != 0,
            temperature_ready: (data[1] & BitFlags::DIE_TEMP_RDY_INT) != 0,
        };
        Ok(status)
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
