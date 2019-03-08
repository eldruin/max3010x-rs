//! Max30102-specific configuration methods.
use crate::{marker, Error, Led, Max3010x, Register as Reg, TimeSlot};
use core::marker::PhantomData;
use hal::blocking::i2c;

impl<I2C, E, MODE> Max3010x<I2C, marker::ic::Max30102, MODE>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
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
            spo2_config: self.spo2_config,
            int_en1: self.int_en1,
            int_en2: self.int_en2,
            _ic: PhantomData,
            _mode: PhantomData,
        };
        Ok(dev)
    }

    /// Change into SpO2 (oximeter) mode.
    ///
    /// This changes the mode and clears the FIFO data.
    pub fn into_oximeter(
        mut self,
    ) -> Result<Max3010x<I2C, marker::ic::Max30102, marker::mode::Oximeter>, Error<E>> {
        let mode = self.mode.with_low(0b0000_0100).with_high(0b0000_0011);
        self.change_mode(mode)?;
        self.clear_fifo()?;
        let dev = Max3010x {
            i2c: self.i2c,
            temperature_measurement_started: self.temperature_measurement_started,
            mode: self.mode,
            fifo_config: self.fifo_config,
            spo2_config: self.spo2_config,
            int_en1: self.int_en1,
            int_en2: self.int_en2,
            _ic: PhantomData,
            _mode: PhantomData,
        };
        Ok(dev)
    }

    /// Change into multi-LED mode.
    ///
    /// This changes the mode and clears the FIFO data.
    pub fn into_multi_led(
        mut self,
    ) -> Result<Max3010x<I2C, marker::ic::Max30102, marker::mode::MultiLED>, Error<E>> {
        let mode = self.mode.with_high(0b0000_0111);
        self.change_mode(mode)?;
        self.clear_fifo()?;
        let dev = Max3010x {
            i2c: self.i2c,
            temperature_measurement_started: self.temperature_measurement_started,
            mode: self.mode,
            fifo_config: self.fifo_config,
            spo2_config: self.spo2_config,
            int_en1: self.int_en1,
            int_en2: self.int_en2,
            _ic: PhantomData,
            _mode: PhantomData,
        };
        Ok(dev)
    }

    /// Set the LED pulse amplitude
    ///
    /// The amplitude value corresponds to a typical current of 0.0 mA for 0
    /// up to 51.0 mA for 255.
    pub fn set_pulse_amplitude(&mut self, led: Led, amplitude: u8) -> Result<(), Error<E>> {
        match led {
            Led::Led1 => self.write_data(&[Reg::LED1_PA, amplitude]),
            Led::Led2 => self.write_data(&[Reg::LED2_PA, amplitude]),
            Led::All => self.write_data(&[Reg::LED1_PA, amplitude, amplitude]),
        }
    }
}

impl TimeSlot {
    fn get_mask(self) -> u8 {
        match self {
            TimeSlot::Disabled => 0,
            TimeSlot::Led1 => 1,
            TimeSlot::Led2 => 2,
        }
    }
}

impl<I2C, E> Max3010x<I2C, marker::ic::Max30102, marker::mode::MultiLED>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Configure LED time slots in Multi-LED mode
    ///
    /// The slots should be activated in order. i.e. slot 2 cannot be
    /// activated if slot 1 is disabled.
    /// Failing to do so will return `Error::InvalidArguments`.
    pub fn set_led_time_slots(&mut self, slots: [TimeSlot; 4]) -> Result<(), Error<E>> {
        use TimeSlot::Disabled;
        let mut last_slot_is_disabled = slots[0] == Disabled;
        for slot in &slots {
            if last_slot_is_disabled && *slot != Disabled {
                return Err(Error::InvalidArguments);
            }
            last_slot_is_disabled = *slot == Disabled;
        }
        let data = [
            Reg::SLOT_CONFIG0,
            slots[1].get_mask() << 4 | slots[0].get_mask(),
            slots[3].get_mask() << 4 | slots[2].get_mask(),
        ];
        self.write_data(&data)
    }
}
