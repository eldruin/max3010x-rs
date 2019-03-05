extern crate embedded_hal_mock as hal;
use hal::i2c::Transaction as I2cTrans;
extern crate max3010x;
use max3010x::{LedPulseWidth as LedPw, SampleRate as SR, SpO2AdcRange as AdcRge};
mod base;
use base::{destroy, new, BitFlags as BF, Register as Reg, DEV_ADDR};

#[test]
fn can_change_into_oximeter() {
    let transactions = [
        I2cTrans::write(DEV_ADDR, vec![Reg::MODE, 0b011]),
        I2cTrans::write(DEV_ADDR, vec![Reg::FIFO_WR_PTR, 0, 0, 0]),
    ];
    let dev = new(&transactions);
    let dev = dev.into_oximeter().unwrap();
    destroy(dev);
}

macro_rules! set_test {
    ($name:ident, $method:ident, $arg:expr, $expected:expr) => {
        set_in_mode_test!(
            $name,
            into_oximeter,
            0b11,
            $method,
            [$arg],
            SPO2_CONFIG,
            $expected
        );
    };
}

set_in_mode_test!(
    enable_new_fifo_data_ready_interrupt,
    into_oximeter,
    0b11,
    enable_new_fifo_data_ready_interrupt,
    [],
    INT_EN1,
    BF::PPG_RDY_INT
);
set_in_mode_test!(
    disable_new_fifo_data_ready_interrupt,
    into_oximeter,
    0b11,
    disable_new_fifo_data_ready_interrupt,
    [],
    INT_EN1,
    0
);

set_test!(adc_rge_2k, set_adc_range, AdcRge::Fs2k, 0);
set_test!(adc_rge_4k, set_adc_range, AdcRge::Fs4k, 1 << 5);
set_test!(adc_rge_8k, set_adc_range, AdcRge::Fs8k, 2 << 5);
set_test!(adc_rge_16k, set_adc_range, AdcRge::Fs16k, 3 << 5);

set_test!(can_set_led_pw_69, set_led_pulse_width, LedPw::Pw69, 0);
set_test!(can_set_led_pw_118, set_led_pulse_width, LedPw::Pw118, 1);
set_test!(can_set_led_pw_215, set_led_pulse_width, LedPw::Pw215, 2);
set_test!(can_set_led_pw_411, set_led_pulse_width, LedPw::Pw411, 3);

set_test!(can_set_sr_50, set_sample_rate, SR::Sps50, 0);
set_test!(can_set_sr_100, set_sample_rate, SR::Sps100, 1 << 2);
set_test!(can_set_sr_200, set_sample_rate, SR::Sps200, 2 << 2);
set_test!(can_set_sr_400, set_sample_rate, SR::Sps400, 3 << 2);
set_test!(can_set_sr_800, set_sample_rate, SR::Sps800, 4 << 2);
set_test!(can_set_sr_1000, set_sample_rate, SR::Sps1000, 5 << 2);
set_test!(can_set_sr_1600, set_sample_rate, SR::Sps1600, 6 << 2);

#[test]
fn cannot_set_sr_3200() {
    // Exemplary integration test. All other combinations tested in unit tests
    let transactions = [
        I2cTrans::write(DEV_ADDR, vec![Reg::MODE, 0b11]),
        I2cTrans::write(DEV_ADDR, vec![Reg::FIFO_WR_PTR, 0, 0, 0]),
    ];
    let dev = new(&transactions);
    let mut dev = dev.into_oximeter().unwrap();
    assert_invalid_args!(dev.set_sample_rate(SR::Sps3200));
    destroy(dev);
}
