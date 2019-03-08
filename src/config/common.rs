//! Common device configuration methods.
use crate::{
    marker, private, AdcRange, BitFlags as BF, Config, Error, FifoAlmostFullLevelInterrupt,
    LedPulseWidth, Max3010x, Register as Reg, SampleAveraging, SampleRate,
};
use hal::blocking::i2c;

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

macro_rules! flip_flag_method_impl {
    ($name:ident, $doc:expr, $reg:ident, $reg_variable:ident, $config_method:ident, $bitflag:ident) => {
        #[doc = $doc]
        pub fn $name(&mut self) -> Result<(), Error<E>> {
            let $reg_variable = self.$reg_variable.$config_method(BF::$bitflag);
            self.write_data(&[Reg::$reg, $reg_variable.bits])?;
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
        let mode = self.mode.with_high(BF::RESET);
        self.write_data(&[Reg::MODE, mode.bits])
    }

    /// Put the device in power-save mode.
    pub fn shutdown(&mut self) -> Result<(), Error<E>> {
        let mode = self.mode.with_high(BF::SHUTDOWN);
        self.change_mode(mode)
    }

    /// Wake up from power-save mode.
    pub fn wake_up(&mut self) -> Result<(), Error<E>> {
        let mode = self.mode.with_low(BF::SHUTDOWN);
        self.change_mode(mode)
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
        self.write_data(&[Reg::FIFO_CONFIG, fifo_config.bits])?;
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
        self.write_data(&[Reg::FIFO_CONFIG, fifo_config.bits])?;
        self.fifo_config = fifo_config;
        Ok(())
    }

    /// Resets the FIFO read and write pointers and overflow counter to 0.
    pub fn clear_fifo(&mut self) -> Result<(), Error<E>> {
        self.write_data(&[Reg::FIFO_WR_PTR, 0, 0, 0])
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

    high_low_flag_impl!(
        enable_fifo_almost_full_interrupt,
        "Enable FIFO almost full interrupt",
        disable_fifo_almost_full_interrupt,
        "Disable FIFO almost full interrupt",
        INT_EN1,
        int_en1,
        FIFO_A_FULL_INT
    );

    high_low_flag_impl!(
        enable_alc_overflow_interrupt,
        "Enable ambient light cancellation overflow interrupt",
        disable_alc_overflow_interrupt,
        "Disable ambient light cancellation overflow interrupt",
        INT_EN1,
        int_en1,
        ALC_OVF_INT
    );

    high_low_flag_impl!(
        enable_temperature_ready_interrupt,
        "Enable internal die temperature conversion ready interrupt",
        disable_temperature_ready_interrupt,
        "Disable internal die temperature conversion ready interrupt",
        INT_EN2,
        int_en2,
        DIE_TEMP_RDY_INT
    );

    pub(crate) fn change_mode(&mut self, mode: Config) -> Result<(), Error<E>> {
        self.write_data(&[Reg::MODE, mode.bits])?;
        self.mode = mode;
        Ok(())
    }
}

#[doc(hidden)]
pub trait ValidateSrPw: private::Sealed {
    /// Check the pulse width and sample rate combination
    fn check<E>(width: LedPulseWidth, rate: SampleRate) -> Result<(), Error<E>>;
}

fn check_red_only<E>(pw: LedPulseWidth, sr: SampleRate) -> Result<(), Error<E>> {
    use LedPulseWidth::*;
    use SampleRate::*;

    if (sr == Sps3200 && (pw == Pw118 || pw == Pw215 || pw == Pw411))
        || (sr == Sps1600 && pw == Pw411)
    {
        Err(Error::InvalidArguments)
    } else {
        Ok(())
    }
}

fn check_red_ir<E>(pw: LedPulseWidth, sr: SampleRate) -> Result<(), Error<E>> {
    use LedPulseWidth::*;
    use SampleRate::*;

    if sr == Sps3200
        || (sr == Sps1600 && (pw == Pw118 || pw == Pw215 || pw == Pw411))
        || (sr == Sps1000 && (pw == Pw215 || pw == Pw411))
        || (sr == Sps800 && pw == Pw411)
    {
        Err(Error::InvalidArguments)
    } else {
        Ok(())
    }
}

impl ValidateSrPw for marker::mode::HeartRate {
    fn check<E>(pw: LedPulseWidth, sr: SampleRate) -> Result<(), Error<E>> {
        check_red_only(pw, sr)
    }
}

impl ValidateSrPw for marker::mode::Oximeter {
    fn check<E>(pw: LedPulseWidth, sr: SampleRate) -> Result<(), Error<E>> {
        check_red_ir(pw, sr)
    }
}

impl ValidateSrPw for marker::mode::MultiLED {
    fn check<E>(_width: LedPulseWidth, _rate: SampleRate) -> Result<(), Error<E>> {
        Ok(())
    }
}

impl<I2C, E, IC, MODE> Max3010x<I2C, IC, MODE>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
    MODE: ValidateSrPw,
{
    fn get_pulse_width(&self) -> LedPulseWidth {
        let pw_bits = self.spo2_config.bits & (BF::LED_PW0 | BF::LED_PW1);
        match pw_bits {
            0 => LedPulseWidth::Pw69,
            1 => LedPulseWidth::Pw118,
            2 => LedPulseWidth::Pw215,
            3 => LedPulseWidth::Pw411,
            _ => unreachable!(),
        }
    }

    fn get_sample_rate(&self) -> SampleRate {
        let sr_bits = (self.spo2_config.bits & (BF::SPO2_SR0 | BF::SPO2_SR1 | BF::SPO2_SR2)) >> 2;
        match sr_bits {
            0 => SampleRate::Sps50,
            1 => SampleRate::Sps100,
            2 => SampleRate::Sps200,
            3 => SampleRate::Sps400,
            4 => SampleRate::Sps800,
            5 => SampleRate::Sps1000,
            6 => SampleRate::Sps1600,
            7 => SampleRate::Sps3200,
            _ => unreachable!(),
        }
    }

    /// Configure the LED pulse width.
    ///
    /// This determines the ADC resolution.
    pub fn set_led_pulse_width(&mut self, width: LedPulseWidth) -> Result<(), Error<E>> {
        use LedPulseWidth::*;
        MODE::check::<E>(width, self.get_sample_rate())?;
        let config = self.spo2_config.with_low(BF::LED_PW0).with_low(BF::LED_PW1);
        let config = match width {
            Pw69 => config,
            Pw118 => config.with_high(BF::LED_PW0),
            Pw215 => config.with_high(BF::LED_PW1),
            Pw411 => config.with_high(BF::LED_PW0).with_high(BF::LED_PW1),
        };
        self.write_data(&[Reg::SPO2_CONFIG, config.bits])?;
        self.spo2_config = config;
        Ok(())
    }

    /// Configure the sample rate
    ///
    /// This depends on the LED pulse width. Calling this with an inappropriate
    /// value for the selected pulse with will return `Error::InvalidArgument`
    pub fn set_sample_rate(&mut self, sample_rate: SampleRate) -> Result<(), Error<E>> {
        use SampleRate::*;
        MODE::check::<E>(self.get_pulse_width(), sample_rate)?;
        let config = self
            .spo2_config
            .with_low(BF::SPO2_SR0)
            .with_low(BF::SPO2_SR1)
            .with_low(BF::SPO2_SR2);
        let config = match sample_rate {
            Sps50 => config,
            Sps100 => config.with_high(BF::SPO2_SR0),
            Sps200 => config.with_high(BF::SPO2_SR1),
            Sps400 => config.with_high(BF::SPO2_SR1).with_high(BF::SPO2_SR0),
            Sps800 => config.with_high(BF::SPO2_SR2),
            Sps1000 => config.with_high(BF::SPO2_SR2).with_high(BF::SPO2_SR0),
            Sps1600 => config.with_high(BF::SPO2_SR2).with_high(BF::SPO2_SR1),
            Sps3200 => config
                .with_high(BF::SPO2_SR2)
                .with_high(BF::SPO2_SR1)
                .with_high(BF::SPO2_SR0),
        };
        self.write_data(&[Reg::SPO2_CONFIG, config.bits])?;
        self.spo2_config = config;
        Ok(())
    }
}

#[doc(hidden)]
pub trait HasDataReadyInterrupt {}

impl HasDataReadyInterrupt for marker::mode::HeartRate {}
impl HasDataReadyInterrupt for marker::mode::Oximeter {}

impl<I2C, E, IC, MODE> Max3010x<I2C, IC, MODE>
where
    I2C: i2c::Write<Error = E>,
    MODE: HasDataReadyInterrupt,
{
    high_low_flag_impl!(
        enable_new_fifo_data_ready_interrupt,
        "Enable new FIFO data ready interrupt",
        disable_new_fifo_data_ready_interrupt,
        "Disable new FIFO data ready interrupt",
        INT_EN1,
        int_en1,
        PPG_RDY_INT
    );
}

impl<I2C, E, IC> Max3010x<I2C, IC, marker::mode::Oximeter>
where
    I2C: i2c::Write<Error = E>,
{
    /// Configure analog-to-digital converter range.
    pub fn set_adc_range(&mut self, range: AdcRange) -> Result<(), Error<E>> {
        use AdcRange::*;
        let new_config = self
            .spo2_config
            .with_low(BF::ADC_RGE0)
            .with_low(BF::ADC_RGE1);
        let new_config = match range {
            Fs2k => new_config,
            Fs4k => new_config.with_high(BF::ADC_RGE0),
            Fs8k => new_config.with_high(BF::ADC_RGE1),
            Fs16k => new_config.with_high(BF::ADC_RGE0).with_high(BF::ADC_RGE1),
        };
        self.write_data(&[Reg::SPO2_CONFIG, new_config.bits])?;
        self.spo2_config = new_config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{check_red_ir, check_red_only, LedPulseWidth as LedPw, SampleRate as SR};

    #[test]
    fn invalid_combinations_oximeter_sample_rate_800_pulse_width_411() {
        check_red_ir::<()>(LedPw::Pw411, SR::Sps800).expect_err("Should return error.");
    }

    #[test]
    fn invalid_combinations_oximeter_sample_rate_1000_pulse_width() {
        check_red_ir::<()>(LedPw::Pw215, SR::Sps1000).expect_err("Should return error.");
        check_red_ir::<()>(LedPw::Pw411, SR::Sps1000).expect_err("Should return error.");
    }
    #[test]
    fn invalid_combinations_oximeter_sample_rate_1600_pulse_width() {
        check_red_ir::<()>(LedPw::Pw118, SR::Sps1600).expect_err("Should return error.");
        check_red_ir::<()>(LedPw::Pw215, SR::Sps1600).expect_err("Should return error.");
        check_red_ir::<()>(LedPw::Pw411, SR::Sps1600).expect_err("Should return error.");
    }
    #[test]
    fn invalid_combinations_oximeter_sample_rate_3200_pulse_width() {
        check_red_ir::<()>(LedPw::Pw69, SR::Sps3200).expect_err("Should return error.");
        check_red_ir::<()>(LedPw::Pw118, SR::Sps3200).expect_err("Should return error.");
        check_red_ir::<()>(LedPw::Pw215, SR::Sps3200).expect_err("Should return error.");
        check_red_ir::<()>(LedPw::Pw411, SR::Sps3200).expect_err("Should return error.");
    }

    #[test]
    fn invalid_combinations_heart_rate_sample_rate_1600_pulse_width_411() {
        check_red_only::<()>(LedPw::Pw411, SR::Sps1600).expect_err("Should return error.");
    }
    #[test]
    fn invalid_combinations_heart_rate_sample_rate_3200_pulse_width() {
        check_red_only::<()>(LedPw::Pw118, SR::Sps3200).expect_err("Should return error.");
        check_red_only::<()>(LedPw::Pw215, SR::Sps3200).expect_err("Should return error.");
        check_red_only::<()>(LedPw::Pw411, SR::Sps3200).expect_err("Should return error.");
    }
}