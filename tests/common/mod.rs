extern crate max3010x;
use self::max3010x::Max3010x;
use hal::i2c::{Mock as I2cMock, Transaction as I2cTrans};

pub const DEV_ADDR: u8 = 0b1010111;

pub struct Register;
#[allow(unused)]
impl Register {
    pub const REV_ID: u8 = 0xFE;
    pub const PART_ID: u8 = 0xFF;
}

pub fn new(transactions: &[I2cTrans]) -> Max3010x<I2cMock> {
    Max3010x::new(I2cMock::new(&transactions))
}

pub fn destroy(sensor: Max3010x<I2cMock>) {
    sensor.destroy().done();
}
