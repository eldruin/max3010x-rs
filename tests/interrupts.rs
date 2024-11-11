extern crate embedded_hal_mock as hal;
use hal::eh1::i2c::Transaction as I2cTrans;
extern crate max3010x;
extern crate nb;
use max3010x::InterruptStatus;
mod base;
use base::{destroy, new, BitFlags as BF, Register as Reg, DEV_ADDR};

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
