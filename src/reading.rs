//! Reading data method implementation.

use super::{
    marker, private, BitFlags, Error, InterruptStatus, LedPulseWidth, Max3010x, Register,
    SamplingRate, DEVICE_ADDRESS,
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
    MODE: ChannelCount<IC, MODE>,
{
    /// Reads samples from FIFO.
    ///
    /// Reads data from the FIFO until all the available samples are read or
    /// the input buffer is full.
    ///
    /// Returns the number of _samples_ read.
    ///
    /// The output buffer must contain one element per channel per sample.
    ///
    /// Note: This method takes care of shifting the data according to the
    /// ADC resolution.
    pub fn read_fifo(&mut self, output_data: &mut [u32]) -> Result<u8, Error<E>> {
        let mode_channels = usize::from(MODE::CHANNEL_COUNT);

        if output_data.len() < mode_channels {
            return Ok(0);
        }
        let samples = self.get_available_sample_count()?;
        let samples_fitting_in_input = output_data.len() / mode_channels;
        let sample_count = core::cmp::min(usize::from(samples), samples_fitting_in_input);
        if sample_count != 0 {
            self.read_samples(sample_count, output_data)?;
        }
        Ok(sample_count as u8) // the maximum is 32 so this is ok
    }

    fn read_samples(&mut self, sample_count: usize, output: &mut [u32]) -> Result<(), Error<E>> {
        const BYTES_PER_SAMPLE: usize = 3;
        const MAX_CHANNEL_COUNT: usize = 2; // for max30102
        const FIFO_SAMPLE_SIZE: usize = 32;

        let mode_channels = usize::from(MODE::CHANNEL_COUNT);
        let sample_shift = self.get_sample_shift();
        let byte_count = sample_count * mode_channels * BYTES_PER_SAMPLE;
        // maximum size (could be optimized by using mode_channels but this
        // needs https://github.com/rust-lang/rust/issues/42863)
        let mut data = [0; FIFO_SAMPLE_SIZE * MAX_CHANNEL_COUNT * BYTES_PER_SAMPLE];
        self.read_data(Register::FIFO_DATA, &mut data[..byte_count])?;
        for (out_idx, out_item) in output
            .iter_mut()
            .enumerate()
            .take(sample_count * mode_channels)
        {
            let sample_idx = out_idx * BYTES_PER_SAMPLE;
            *out_item = (u32::from(data[sample_idx]) << 16
                | u32::from(data[sample_idx + 1]) << 8
                | u32::from(data[sample_idx + 2]))
                >> sample_shift;
        }
        Ok(())
    }

    fn get_sample_shift(&self) -> usize {
        match self.get_pulse_width() {
            LedPulseWidth::Pw69 => 3,
            LedPulseWidth::Pw118 => 2,
            LedPulseWidth::Pw215 => 1,
            LedPulseWidth::Pw411 => 0,
        }
    }
}

impl<I2C, IC, MODE> Max3010x<I2C, IC, MODE> {
    pub(crate) fn get_pulse_width(&self) -> LedPulseWidth {
        let pw_bits = self.spo2_config.bits & (BitFlags::LED_PW0 | BitFlags::LED_PW1);
        match pw_bits {
            0 => LedPulseWidth::Pw69,
            1 => LedPulseWidth::Pw118,
            2 => LedPulseWidth::Pw215,
            3 => LedPulseWidth::Pw411,
            _ => unreachable!(),
        }
    }

    pub(crate) fn get_sampling_rate(&self) -> SamplingRate {
        convert_sampling_rate(self.spo2_config.bits)
    }
}

fn convert_sampling_rate(spo2_config: u8) -> SamplingRate {
    let sr_bits =
        (spo2_config & (BitFlags::SPO2_SR0 | BitFlags::SPO2_SR1 | BitFlags::SPO2_SR2)) >> 2;
    match sr_bits {
        0 => SamplingRate::Sps50,
        1 => SamplingRate::Sps100,
        2 => SamplingRate::Sps200,
        3 => SamplingRate::Sps400,
        4 => SamplingRate::Sps800,
        5 => SamplingRate::Sps1000,
        6 => SamplingRate::Sps1600,
        7 => SamplingRate::Sps3200,
        _ => unreachable!(),
    }
}

impl<I2C, E, IC, MODE> Max3010x<I2C, IC, MODE>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Get number of samples available for reading from FIFO.
    pub fn get_available_sample_count(&mut self) -> Result<u8, Error<E>> {
        let mut data = [0; 3];
        self.read_data(Register::FIFO_WR_PTR, &mut data)?;
        let wr_ptr = data[0] & 0x1F;
        let rd_ptr = data[2] & 0x1F;
        let has_rolled_over = rd_ptr > wr_ptr;
        if has_rolled_over {
            Ok(32 - rd_ptr + wr_ptr)
        } else {
            Ok(wr_ptr - rd_ptr)
        }
    }

    /// Get number of samples lost from FIFO.
    ///
    /// If FIFO rollover is not enabled, when the FIFO is full the samples are
    /// not pushed on to the FIFO.
    pub fn get_overflow_sample_count(&mut self) -> Result<u8, Error<E>> {
        let v = self.read_register(Register::OVF_COUNTER)?;
        Ok(v & 0x1F)
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
            self.write_data(&[Register::TEMP_CONFIG, BitFlags::TEMP_EN])
                .map_err(nb::Error::Other)?;
            self.temperature_measurement_started = true;
            Err(nb::Error::WouldBlock)
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

#[cfg(test)]
mod convert_sampling_rate_tests {
    use super::{convert_sampling_rate, SamplingRate};
    macro_rules! convert_sampling_rate_test {
        ($name:ident, $bits:expr, $rate:ident) => {
            #[test]
            fn $name() {
                let rate = convert_sampling_rate($bits);
                assert_eq!(SamplingRate::$rate, rate);
            }
        };
    }

    convert_sampling_rate_test!(sps50, 0 << 2, Sps50);
    convert_sampling_rate_test!(sps100, 1 << 2, Sps100);
    convert_sampling_rate_test!(sps200, 2 << 2, Sps200);
    convert_sampling_rate_test!(sps400, 3 << 2, Sps400);
    convert_sampling_rate_test!(sps800, 4 << 2, Sps800);
    convert_sampling_rate_test!(sps1000, 5 << 2, Sps1000);
    convert_sampling_rate_test!(sps1600, 6 << 2, Sps1600);
    convert_sampling_rate_test!(sps3200, 7 << 2, Sps3200);
    convert_sampling_rate_test!(other_bits_are_ignored, 0b1110_0011, Sps50);
}
