extern crate embedded_hal_mock as hal;
use hal::i2c::Transaction as I2cTrans;
mod common;
use common::{destroy, new, Register as Reg, DEV_ADDR};

#[test]
fn can_get_revision_id() {
    let transactions = [I2cTrans::write_read(
        DEV_ADDR,
        vec![Reg::REV_ID],
        vec![0xAB],
    )];
    let mut dev = new(&transactions);
    let id = dev.get_revision_id().unwrap();
    assert_eq!(0xAB, id);
    destroy(dev);
}

#[test]
fn can_get_part_id() {
    let transactions = [I2cTrans::write_read(
        DEV_ADDR,
        vec![Reg::PART_ID],
        vec![0xAB],
    )];
    let mut dev = new(&transactions);
    let id = dev.get_part_id().unwrap();
    assert_eq!(0xAB, id);
    destroy(dev);
}
