extern crate max3010x;
use self::max3010x::Max3010x;
use hal::i2c::{Mock as I2cMock, Transaction as I2cTrans};

pub const DEV_ADDR: u8 = 0b101_0111;

pub struct Register;
#[allow(unused)]
impl Register {
    pub const FIFO_WR_PTR: u8 = 0x04;
    pub const MODE: u8 = 0x09;
    pub const TEMP_INT: u8 = 0x1F;
    pub const TEMP_CONFIG: u8 = 0x21;
    pub const REV_ID: u8 = 0xFE;
    pub const PART_ID: u8 = 0xFF;
}

pub struct BitFlags;
#[allow(unused)]
impl BitFlags {
    pub const TEMP_EN: u8 = 0x01;
    pub const SHUTDOWN: u8 = 0b1000_0000;
}

pub fn new(transactions: &[I2cTrans]) -> Max3010x<I2cMock> {
    Max3010x::new(I2cMock::new(&transactions))
}

pub fn destroy(sensor: Max3010x<I2cMock>) {
    sensor.destroy().done();
}

#[macro_export]
macro_rules! assert_would_block {
    ($result:expr) => {
        match $result {
            Err(nb::Error::WouldBlock) => (),
            _ => panic!("Did not return nb::Error::WouldBlock"),
        }
    };
}

 #[macro_export]
 macro_rules! assert_near {
     ($left:expr, $right:expr, $eps:expr) => {
        assert!(($left - $right) < $eps && ($right - $left) < $eps);
     };
 }
