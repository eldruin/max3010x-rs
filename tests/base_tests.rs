extern crate embedded_hal_mock as hal;
extern crate max3010x;
use max3010x::Error;

#[allow(unused)]
mod base;

#[test]
fn assert_would_block_can_succeed() {
    assert_would_block!(Err::<(), nb::Error<()>>(nb::Error::WouldBlock));
}

#[test]
#[should_panic]
fn assert_would_block_can_fail() {
    assert_would_block!(Ok::<(), nb::Error<()>>(()));
}

#[test]
fn assert_invalid_args_can_succeed() {
    assert_invalid_args!(Err::<(), Error<()>>(Error::InvalidArguments));
}

#[test]
#[should_panic]
fn assert_invalid_args_can_fail() {
    assert_invalid_args!(Ok::<(), Error<()>>(()));
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
