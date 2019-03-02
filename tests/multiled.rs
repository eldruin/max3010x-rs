extern crate embedded_hal_mock as hal;
use hal::i2c::Transaction as I2cTrans;
extern crate max3010x;
extern crate nb;
use max3010x::TimeSlot;
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
