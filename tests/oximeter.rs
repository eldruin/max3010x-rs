extern crate embedded_hal_mock as hal;
use hal::i2c::Transaction as I2cTrans;
extern crate max3010x;
mod common;
use common::{destroy, new, Register as Reg, DEV_ADDR};


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
