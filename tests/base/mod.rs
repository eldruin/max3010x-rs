extern crate embedded_hal_mock as hal;
extern crate max3010x;
use self::hal::i2c::{Mock as I2cMock, Transaction as I2cTrans};
use self::max3010x::{marker, Max3010x};

pub const DEV_ADDR: u8 = 0b101_0111;

pub struct Register;
#[allow(unused)]
impl Register {
    pub const INT_STATUS: u8 = 0x0;
    pub const INT_EN1: u8 = 0x02;
    pub const INT_EN2: u8 = 0x03;
    pub const FIFO_WR_PTR: u8 = 0x04;
    pub const OVF_COUNTER: u8 = 0x05;
    pub const FIFO_DATA: u8 = 0x07;
    pub const FIFO_CONFIG: u8 = 0x08;
    pub const MODE: u8 = 0x09;
    pub const SPO2_CONFIG: u8 = 0x0A;
    pub const LED1_PA: u8 = 0x0C;
    pub const LED2_PA: u8 = 0x0D;
    pub const SLOT_CONFIG0: u8 = 0x11;
    pub const TEMP_INT: u8 = 0x1F;
    pub const TEMP_CONFIG: u8 = 0x21;
    pub const REV_ID: u8 = 0xFE;
    pub const PART_ID: u8 = 0xFF;
}

pub struct BitFlags;
#[allow(unused)]
impl BitFlags {
    pub const FIFO_A_FULL_INT: u8 = 0b1000_0000;
    pub const ALC_OVF_INT: u8 = 0b0010_0000;
    pub const DIE_TEMP_RDY_INT: u8 = 0b0000_0010;
    pub const PPG_RDY_INT: u8 = 0b0100_0000;
    pub const PWR_RDY_INT: u8 = 0b0000_0001;
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

#[macro_export]
macro_rules! assert_invalid_args {
    ($result:expr) => {
        match $result {
            Err(max3010x::Error::InvalidArguments) => (),
            _ => panic!("Did not return Error::InvalidArguments"),
        }
    };
}

#[macro_export]
macro_rules! assert_near {
    ($left:expr, $right:expr, $eps:expr) => {
        assert!(($left - $right) < $eps && ($right - $left) < $eps);
    };
}

#[macro_export]
macro_rules! read_test {
    ($name:ident, $method:ident, [$($arg:expr),*], $reg:ident, [$($values:expr),*], $expected:expr) => {
        #[test]
        fn $name() {
            let transactions = [
                I2cTrans::write_read(
                    DEV_ADDR,
                    vec![Reg::$reg],
                    vec![$($values),*]
                )
            ];
            let mut dev = new (&transactions);
            let result = dev.$method($($arg),*).unwrap();
            assert_eq!(result, $expected);
            destroy(dev);
        }
    };
}

#[macro_export]
macro_rules! write_test {
    ($name:ident, $method:ident, [$($arg:expr),*], $reg:ident, [$($values:expr),*]) => {
        #[test]
        fn $name() {
            let transactions = [I2cTrans::write(DEV_ADDR, vec![Reg::$reg, $($values),*])];
            let mut dev = new (&transactions);
            dev.$method($($arg),*).unwrap();
            destroy(dev);
        }
    };
}

#[macro_export]
macro_rules! high_low_flag_method_test {
    ($method_en:ident, $expected_en:expr, $method_dis:ident, $expected_dis:expr, $reg:ident) => {
        write_test!($method_en, $method_en, [], $reg, [$expected_en]);
        write_test!($method_dis, $method_dis, [], $reg, [$expected_dis]);
    };
}

#[macro_export]
macro_rules! set_in_mode_test {
    ($name:ident, $mode_method:ident, $mode:expr, $method:ident, $arg:expr,
     $reg:ident, $expected:expr) => {
        #[test]
        fn $name() {
            let transactions = [
                I2cTrans::write(DEV_ADDR, vec![Reg::MODE, $mode]),
                I2cTrans::write(DEV_ADDR, vec![Reg::FIFO_WR_PTR, 0, 0, 0]),
                I2cTrans::write(DEV_ADDR, vec![Reg::$reg, $expected]),
            ];
            let dev = new(&transactions);
            let mut dev = dev.$mode_method().unwrap();
            dev.$method($arg).unwrap();
            destroy(dev);
        }
    };
}

#[macro_export]
macro_rules! set_led_pw_test {
    ($name:ident, $mode_method:ident, $mode:expr, $width:expr, $expected:expr) => {
        set_in_mode_test!(
            $name,
            $mode_method,
            $mode,
            set_led_pulse_width,
            $width,
            SPO2_CONFIG,
            $expected
        );
    };
}

#[macro_export]
macro_rules! set_sample_rate_test {
    ($name:ident, $mode_method:ident, $mode:expr, $width:expr, $expected:expr) => {
        set_in_mode_test!(
            $name,
            $mode_method,
            $mode,
            set_sample_rate,
            $width,
            SPO2_CONFIG,
            $expected
        );
    };
}
