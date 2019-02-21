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

macro_rules! available_sample_count_test {
    ($name:ident, $wr_ptr:expr, $rd_ptr:expr, $expected:expr) => (
        #[test]
        fn $name() {
            let transactions = [I2cTrans::write_read(
                DEV_ADDR,
                vec![Reg::FIFO_WR_PTR],
                vec![$wr_ptr, 0, $rd_ptr],
            )];
            let mut dev = new(&transactions);
            let count = dev.get_available_sample_count().unwrap();
            assert_eq!($expected, count);
            destroy(dev);
        }
    )
}

mod available_sample_count {
    use super::*;
    available_sample_count_test!(zero, 0, 0, 0);
    available_sample_count_test!(one, 1, 0, 1);
    available_sample_count_test!(two, 2, 0, 2);
    available_sample_count_test!(rollover, 0, 1, 31);
}
