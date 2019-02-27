extern crate embedded_hal_mock as hal;
use hal::i2c::Transaction as I2cTrans;
extern crate max3010x;
extern crate nb;
use max3010x::{FifoAlmostFullLevelInterrupt, InterruptStatus, Led, SampleAveraging};
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

macro_rules! sample_avg_test {
    ($name:ident, $variant:ident, $expected:expr) => {
        write_test!(
            $name,
            set_sample_averaging,
            [SampleAveraging::$variant],
            FIFO_CONFIG,
            [$expected]
        );
    };
}

sample_avg_test!(sample_avg_1, Sa1, 0);
sample_avg_test!(sample_avg_2, Sa2, 0b0010_0000);
sample_avg_test!(sample_avg_4, Sa4, 0b0100_0000);
sample_avg_test!(sample_avg_8, Sa8, 0b0110_0000);
sample_avg_test!(sample_avg_16, Sa16, 0b1000_0000);
sample_avg_test!(sample_avg_32, Sa32, 0b1010_0000);

macro_rules! fifo_a_full_test {
    ($name:ident, $variant:ident, $expected:expr) => {
        write_test!(
            $name,
            set_fifo_almost_full_level_interrupt,
            [FifoAlmostFullLevelInterrupt::$variant],
            FIFO_CONFIG,
            [$expected]
        );
    };
}

fifo_a_full_test!(fifo_a_full_0, L0, 0);
fifo_a_full_test!(fifo_a_full_1, L1, 1);
fifo_a_full_test!(fifo_a_full_2, L2, 2);
fifo_a_full_test!(fifo_a_full_3, L3, 3);
fifo_a_full_test!(fifo_a_full_4, L4, 4);
fifo_a_full_test!(fifo_a_full_5, L5, 5);
fifo_a_full_test!(fifo_a_full_6, L6, 6);
fifo_a_full_test!(fifo_a_full_7, L7, 7);
fifo_a_full_test!(fifo_a_full_8, L8, 8);
fifo_a_full_test!(fifo_a_full_9, L9, 9);
fifo_a_full_test!(fifo_a_full_10, L10, 10);
fifo_a_full_test!(fifo_a_full_11, L11, 11);
fifo_a_full_test!(fifo_a_full_12, L12, 12);
fifo_a_full_test!(fifo_a_full_13, L13, 13);
fifo_a_full_test!(fifo_a_full_14, L14, 14);
fifo_a_full_test!(fifo_a_full_15, L15, 15);

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
    BF::FIFO_A_FULL_INT,
    disable_fifo_almost_full_interrupt,
    0,
    INT_EN1
);

high_low_flag_method_test!(
    enable_alc_overflow_interrupt,
    BF::ALC_OVF_INT,
    disable_alc_overflow_interrupt,
    0,
    INT_EN1
);

high_low_flag_method_test!(
    enable_temperature_ready_interrupt,
    BF::DIE_TEMP_RDY_INT,
    disable_temperature_ready_interrupt,
    0,
    INT_EN2
);

fn is_int_status_eq(a: InterruptStatus, b: InterruptStatus) {
    if a.power_ready != b.power_ready
        || a.fifo_almost_full != b.fifo_almost_full
        || a.new_fifo_data_ready != b.new_fifo_data_ready
        || a.alc_overflow != b.alc_overflow
        || a.temperature_ready != b.temperature_ready
    {
        panic!("Interrupt status is not equal");
    }
}

fn new_int_status(
    power_ready: bool,
    fifo_almost_full: bool,
    new_fifo_data_ready: bool,
    alc_overflow: bool,
    temperature_ready: bool,
) -> InterruptStatus {
    InterruptStatus {
        power_ready,
        fifo_almost_full,
        new_fifo_data_ready,
        alc_overflow,
        temperature_ready,
    }
}

#[test]
fn int_status_is_equal() {
    let a = new_int_status(false, false, false, false, false);
    let b = new_int_status(false, false, false, false, false);
    is_int_status_eq(a, b);
}

#[test]
#[should_panic]
fn int_status_is_not_equal() {
    let a = new_int_status(false, false, false, false, false);
    let b = new_int_status(true, false, false, false, false);
    is_int_status_eq(a, b);
}

macro_rules! int_status_test {
    ($name:ident, [$($values:expr),*], $expected:expr) => {
        #[test]
        fn $name() {
            let transactions = [
                I2cTrans::write_read(
                    DEV_ADDR,
                    vec![Reg::INT_STATUS],
                    vec![$($values),*]
                )
            ];
            let mut dev = new (&transactions);
            let result = dev.read_interrupt_status().unwrap();
            is_int_status_eq(result, $expected);
            destroy(dev);
        }
    };
}

int_status_test!(
    read_int_status_pwr_rdy_false,
    [0, 0],
    new_int_status(false, false, false, false, false)
);

int_status_test!(
    read_int_status_pwr_rdy_true,
    [BF::PWR_RDY_INT, 0],
    new_int_status(true, false, false, false, false)
);

int_status_test!(
    read_int_status_fifo_a_full_false,
    [0, 0],
    new_int_status(false, false, false, false, false)
);

int_status_test!(
    read_int_status_fifo_a_full_true,
    [BF::FIFO_A_FULL_INT, 0],
    new_int_status(false, true, false, false, false)
);

int_status_test!(
    read_int_status_ppg_rdy_false,
    [0, 0],
    new_int_status(false, false, false, false, false)
);

int_status_test!(
    read_int_status_ppg_rdy_true,
    [BF::PPG_RDY_INT, 0],
    new_int_status(false, false, true, false, false)
);

int_status_test!(
    read_int_status_alc_ovf_false,
    [0, 0],
    new_int_status(false, false, false, false, false)
);

int_status_test!(
    read_int_status_alc_ovf_true,
    [BF::ALC_OVF_INT, 0],
    new_int_status(false, false, false, true, false)
);

int_status_test!(
    read_int_status_temp_rdy_false,
    [0, 0],
    new_int_status(false, false, false, false, false)
);

int_status_test!(
    read_int_status_temp_rdy_true,
    [0, BF::DIE_TEMP_RDY_INT],
    new_int_status(false, false, false, false, true)
);

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
