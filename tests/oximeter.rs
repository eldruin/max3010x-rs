extern crate embedded_hal_mock as hal;
use hal::i2c::Transaction as I2cTrans;
extern crate max3010x;
use max3010x::{LedPulseWidth as LedPw, SampleRate as SR, SpO2ADCRange};
mod base;
use base::{destroy, new, Register as Reg, DEV_ADDR};

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

macro_rules! set_adc_range {
    ($name:ident, $variant:ident, $expected:expr) => {
        #[test]
        fn $name() {
            let transactions = [
                I2cTrans::write(DEV_ADDR, vec![Reg::MODE, 0b011]),
                I2cTrans::write(DEV_ADDR, vec![Reg::FIFO_WR_PTR, 0, 0, 0]),
                I2cTrans::write(DEV_ADDR, vec![Reg::SPO2_CONFIG, $expected]),
            ];
            let dev = new(&transactions);
            let mut dev = dev.into_oximeter().unwrap();
            dev.set_adc_range(SpO2ADCRange::$variant).unwrap();
            destroy(dev);
        }
    };
}

set_adc_range!(adc_rge_2k, Fs2k, 0);
set_adc_range!(adc_rge_4k, Fs4k, 1 << 5);
set_adc_range!(adc_rge_8k, Fs8k, 2 << 5);
set_adc_range!(adc_rge_16k, Fs16k, 3 << 5);

set_led_pw_test!(can_set_led_pw_69, into_oximeter, 0b11, LedPw::Pw69, 0);
set_led_pw_test!(can_set_led_pw_118, into_oximeter, 0b11, LedPw::Pw118, 1);
set_led_pw_test!(can_set_led_pw_215, into_oximeter, 0b11, LedPw::Pw215, 2);
set_led_pw_test!(can_set_led_pw_411, into_oximeter, 0b11, LedPw::Pw411, 3);

set_sample_rate_test!(can_set_sr_50, into_oximeter, 0b11, SR::Sps50, 0);
set_sample_rate_test!(can_set_sr_100, into_oximeter, 0b11, SR::Sps100, 1 << 2);
set_sample_rate_test!(can_set_sr_200, into_oximeter, 0b11, SR::Sps200, 2 << 2);
set_sample_rate_test!(can_set_sr_400, into_oximeter, 0b11, SR::Sps400, 3 << 2);
set_sample_rate_test!(can_set_sr_800, into_oximeter, 0b11, SR::Sps800, 4 << 2);
set_sample_rate_test!(can_set_sr_1000, into_oximeter, 0b11, SR::Sps1000, 5 << 2);
set_sample_rate_test!(can_set_sr_1600, into_oximeter, 0b11, SR::Sps1600, 6 << 2);

#[test]
fn cannot_set_sr_3200() {
    // Exemplary test. All other combinations tested in unit tests
    let transactions = [
        I2cTrans::write(DEV_ADDR, vec![Reg::MODE, 0b11]),
        I2cTrans::write(DEV_ADDR, vec![Reg::FIFO_WR_PTR, 0, 0, 0]),
    ];
    let dev = new(&transactions);
    let mut dev = dev.into_oximeter().unwrap();
    assert_invalid_args!(dev.set_sample_rate(SR::Sps3200));
    destroy(dev);
}
