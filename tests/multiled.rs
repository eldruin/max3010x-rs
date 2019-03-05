extern crate embedded_hal_mock as hal;
use hal::i2c::Transaction as I2cTrans;
extern crate max3010x;
extern crate nb;
use max3010x::{LedPulseWidth as LedPw, SampleRate as SR, TimeSlot};
mod base;
use base::{destroy, new, Register as Reg, DEV_ADDR};

#[test]
fn can_change_into_multi_led() {
    let transactions = [
        I2cTrans::write(DEV_ADDR, vec![Reg::MODE, 0b111]),
        I2cTrans::write(DEV_ADDR, vec![Reg::FIFO_WR_PTR, 0, 0, 0]),
    ];
    let dev = new(&transactions);
    let dev = dev.into_multi_led().unwrap();
    destroy(dev);
}

fn cannot_enable_led_slots(slots: [TimeSlot; 4]) {
    let transactions = [
        I2cTrans::write(DEV_ADDR, vec![Reg::MODE, 0b111]),
        I2cTrans::write(DEV_ADDR, vec![Reg::FIFO_WR_PTR, 0, 0, 0]),
    ];
    let dev = new(&transactions);
    let mut dev = dev.into_multi_led().unwrap();
    assert_invalid_args!(dev.set_led_time_slots(slots));
    destroy(dev);
}

#[test]
fn cannot_enable_led_slots_enabled_after_disabled0() {
    use TimeSlot::*;
    cannot_enable_led_slots([Disabled, Led1, Led1, Led1]);
}

#[test]
fn cannot_enable_led_slots_enabled_after_disabled2() {
    use TimeSlot::*;
    cannot_enable_led_slots([Led1, Led1, Disabled, Led1]);
}

fn can_set_led_slots(slots: [TimeSlot; 4], expected: Vec<u8>) {
    let transactions = [
        I2cTrans::write(DEV_ADDR, vec![Reg::MODE, 0b111]),
        I2cTrans::write(DEV_ADDR, vec![Reg::FIFO_WR_PTR, 0, 0, 0]),
        I2cTrans::write(DEV_ADDR, expected),
    ];
    let dev = new(&transactions);
    let mut dev = dev.into_multi_led().unwrap();
    dev.set_led_time_slots(slots).unwrap();
    destroy(dev);
}

#[test]
fn can_set_all_led_slots() {
    use TimeSlot::*;
    can_set_led_slots(
        [Led1, Led2, Led1, Led2],
        vec![Reg::SLOT_CONFIG0, 2 << 4 | 1, 2 << 4 | 1],
    );
}

#[test]
fn can_set_led_slots_with_last_disabled() {
    use TimeSlot::*;
    can_set_led_slots(
        [Led1, Led2, Led1, Disabled],
        vec![Reg::SLOT_CONFIG0, 2 << 4 | 1, 1],
    );
}

#[test]
fn can_set_only_one_led_slots() {
    use TimeSlot::*;
    can_set_led_slots(
        [Led1, Disabled, Disabled, Disabled],
        vec![Reg::SLOT_CONFIG0, 1, 0],
    );
}

macro_rules! set_test {
    ($name:ident, $method:ident, $arg:expr, $expected:expr) => {
        set_in_mode_test!(
            $name,
            into_multi_led,
            0b111,
            $method,
            [$arg],
            SPO2_CONFIG,
            $expected
        );
    };
}

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
set_test!(can_set_sr_3200, set_sample_rate, SR::Sps3200, 7 << 2);
