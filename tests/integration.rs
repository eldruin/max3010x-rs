extern crate embedded_hal_mock as hal;
use hal::i2c::Transaction as I2cTrans;
extern crate max3010x;
extern crate nb;
use max3010x::Led;
mod common;
use common::{destroy, new, BitFlags as BF, Register as Reg, DEV_ADDR};

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

read_test!(can_get_rev_id, get_revision_id, [], REV_ID, [0xAB], 0xAB);
read_test!(can_get_part_id, get_part_id, [], PART_ID, [0xAB], 0xAB);

macro_rules! available_sample_count_test {
    ($name:ident, $wr_ptr:expr, $rd_ptr:expr, $expected:expr) => {
        read_test!(
            $name,
            get_available_sample_count,
            [],
            FIFO_WR_PTR,
            [$wr_ptr, 0, $rd_ptr],
            $expected
        );
    };
}

mod available_sample_count {
    use super::*;
    available_sample_count_test!(zero, 0, 0, 0);
    available_sample_count_test!(one, 1, 0, 1);
    available_sample_count_test!(two, 2, 0, 2);
    available_sample_count_test!(rollover, 0, 1, 31);
}

#[test]
fn can_start_temp_conversion() {
    let transactions = [
        I2cTrans::write_read(DEV_ADDR, vec![Reg::TEMP_CONFIG], vec![0]),
        I2cTrans::write(DEV_ADDR, vec![Reg::TEMP_CONFIG, BF::TEMP_EN]),
    ];
    let mut dev = new(&transactions);
    let result = dev.read_temperature();
    assert_would_block!(result);
    destroy(dev);
}

#[test]
fn blocks_until_temp_ready() {
    let transactions = [I2cTrans::write_read(
        DEV_ADDR,
        vec![Reg::TEMP_CONFIG],
        vec![BF::TEMP_EN],
    )];
    let mut dev = new(&transactions);
    let result = dev.read_temperature();
    assert_would_block!(result);
    destroy(dev);
}

#[test]
fn can_read_temperature() {
    let transactions = [
        I2cTrans::write_read(DEV_ADDR, vec![Reg::TEMP_CONFIG], vec![0]),
        I2cTrans::write(DEV_ADDR, vec![Reg::TEMP_CONFIG, BF::TEMP_EN]),
        I2cTrans::write_read(DEV_ADDR, vec![Reg::TEMP_CONFIG], vec![0]),
        I2cTrans::write_read(DEV_ADDR, vec![Reg::TEMP_INT], vec![-128_i8 as u8, 8]),
    ];
    let mut dev = new(&transactions);
    assert_would_block!(dev.read_temperature());
    let result = dev.read_temperature().unwrap();
    assert_near!(-127.5, result, 0.2);
    destroy(dev);
}

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

write_test!(can_shutdown, shutdown, [], MODE, [BF::SHUTDOWN]);
write_test!(can_wake_up, wake_up, [], MODE, [0]);
write_test!(can_reset, reset, [], MODE, [BF::RESET]);
write_test!(can_clear_fifo, clear_fifo, [], FIFO_WR_PTR, [0, 0, 0]);

#[test]
fn can_change_into_hr() {
    let transactions = [
        I2cTrans::write(DEV_ADDR, vec![Reg::MODE, 0b010]),
        I2cTrans::write(DEV_ADDR, vec![Reg::FIFO_WR_PTR, 0, 0, 0]),
    ];
    let dev = new(&transactions);
    let dev = dev.into_heart_rate().unwrap();
    destroy(dev);
}

#[test]
fn read_fifo_too_short_input_returns0() {
    let transactions = [
        I2cTrans::write(DEV_ADDR, vec![Reg::MODE, 0b010]),
        I2cTrans::write(DEV_ADDR, vec![Reg::FIFO_WR_PTR, 0, 0, 0]),
    ];
    let dev = new(&transactions);

    let mut data = [0; 2];
    let mut dev = dev.into_heart_rate().unwrap();
    let result = dev.read_fifo(&mut data).unwrap();
    assert_eq!(0, result);
    destroy(dev);
}

#[test]
fn read_fifo_no_data_returns0() {
    let transactions = [
        I2cTrans::write(DEV_ADDR, vec![Reg::MODE, 0b010]),
        I2cTrans::write(DEV_ADDR, vec![Reg::FIFO_WR_PTR, 0, 0, 0]),
        I2cTrans::write_read(DEV_ADDR, vec![Reg::FIFO_WR_PTR], vec![0, 0, 0]),
    ];
    let dev = new(&transactions);

    let mut data = [0; 15];
    let mut dev = dev.into_heart_rate().unwrap();
    let result = dev.read_fifo(&mut data).unwrap();
    assert_eq!(0, result);
    destroy(dev);
}

#[test]
fn read_fifo_read_samples() {
    let transactions = [
        I2cTrans::write(DEV_ADDR, vec![Reg::MODE, 0b010]),
        I2cTrans::write(DEV_ADDR, vec![Reg::FIFO_WR_PTR, 0, 0, 0]),
        I2cTrans::write_read(DEV_ADDR, vec![Reg::FIFO_WR_PTR], vec![2, 0, 0]),
        I2cTrans::write_read(DEV_ADDR, vec![Reg::FIFO_DATA], vec![1, 2, 3, 4, 5, 6]),
    ];
    let dev = new(&transactions);

    let mut data = [0; 6];
    let mut dev = dev.into_heart_rate().unwrap();
    let result = dev.read_fifo(&mut data).unwrap();
    assert_eq!(2, result);
    assert_eq!([1, 2, 3, 4, 5, 6], data);
    destroy(dev);
}

mod set_pulse_amplitude {
    use super::*;
    write_test!(led1, set_pulse_amplitude, [Led::Led1, 50], LED1_PA, [50]);
    write_test!(led2, set_pulse_amplitude, [Led::Led2, 50], LED2_PA, [50]);
    write_test!(all, set_pulse_amplitude, [Led::All, 50], LED1_PA, [50, 50]);
}

macro_rules! high_low_flag_method_test {
    ($method_en:ident, $expected_en:expr, $method_dis:ident, $expected_dis:expr, $reg:ident) => {
        write_test!($method_en, $method_en, [], $reg, [$expected_en]);
        write_test!($method_dis, $method_dis, [], $reg, [$expected_dis]);
    };
}

high_low_flag_method_test!(
    enable_fifo_rollover,
    BF::FIFO_ROLLOVER_EN,
    disable_fifo_rollover,
    0,
    FIFO_CONFIG
);

high_low_flag_method_test!(
    enable_fifo_almost_full_interrupt,
    BF::FIFO_A_FULL_INT_EN,
    disable_fifo_almost_full_interrupt,
    0,
    INT_EN1
);

high_low_flag_method_test!(
    enable_alc_overflow_interrupt,
    BF::ALC_OVF_INT_EN,
    disable_alc_overflow_interrupt,
    0,
    INT_EN1
);
