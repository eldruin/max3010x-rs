extern crate max3010x;
use self::max3010x::{marker, Max3010x};
extern crate embedded_hal_mock as hal;
use hal::i2c::{Mock as I2cMock, Transaction as I2cTrans};

pub const DEV_ADDR: u8 = 0b101_0111;

pub struct Register;
#[allow(unused)]
impl Register {
    pub const INT_STATUS: u8 = 0x0;
    pub const INT_EN1: u8 = 0x02;
    pub const INT_EN2: u8 = 0x03;
    pub const FIFO_WR_PTR: u8 = 0x04;
    pub const FIFO_DATA: u8 = 0x07;
    pub const FIFO_CONFIG: u8 = 0x08;
    pub const MODE: u8 = 0x09;
    pub const LED1_PA: u8 = 0x0C;
    pub const LED2_PA: u8 = 0x0D;
    pub const TEMP_INT: u8 = 0x1F;
    pub const TEMP_CONFIG: u8 = 0x21;
    pub const REV_ID: u8 = 0xFE;
    pub const PART_ID: u8 = 0xFF;
}

pub struct BitFlags;
#[allow(unused)]
impl BitFlags {
    pub const FIFO_A_FULL_INT_EN: u8 = 0b1000_0000;
    pub const ALC_OVF_INT_EN: u8 = 0b0010_0000;
    pub const DIE_TEMP_RDY_INT_EN: u8 = 0b0000_0010;
    pub const PPG_RDY: u8 = 0b0100_0000;
    pub const ALC_OVF: u8 = 0b0010_0000;
    pub const PWR_RDY: u8 = 0b0000_0001;
    pub const FIFO_A_FULL: u8 = 0b1000_0000;
    pub const TEMP_EN: u8 = 0x01;
    pub const SHUTDOWN: u8 = 0b1000_0000;
    pub const RESET: u8 = 0b0100_0000;
    pub const FIFO_ROLLOVER_EN: u8 = 0b0001_0000;
}

pub fn new(
    transactions: &[I2cTrans],
) -> Max3010x<I2cMock, marker::ic::Max30102, marker::mode::None> {
    Max3010x::new_max30102(I2cMock::new(&transactions))
}

pub fn destroy<IC, MODE>(sensor: Max3010x<I2cMock, IC, MODE>) {
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

#[test]
fn assert_would_block_can_succeed() {
    assert_would_block!(Err::<(), nb::Error<()>>(nb::Error::WouldBlock));
}

#[test]
#[should_panic]
fn assert_would_block_can_fail() {
    assert_would_block!(Ok::<(), nb::Error<()>>(()));
}

#[macro_export]
macro_rules! assert_near {
    ($left:expr, $right:expr, $eps:expr) => {
        assert!(($left - $right) < $eps && ($right - $left) < $eps);
    };
}

#[test]
fn assert_near_can_succeed() {
    assert_near!(1.0, 1.01, 0.1);
}

#[test]
#[should_panic]
fn assert_near_can_fail() {
    assert_near!(1.0, 4.0, 0.1);
}
