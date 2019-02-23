extern crate embedded_hal;
extern crate linux_embedded_hal;
extern crate max3010x;

use linux_embedded_hal::I2cdev;
use max3010x::Max3010x;

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let mut sensor = Max3010x::new_max30102(dev);
    let part_id = sensor.get_part_id().unwrap();

    // This should print "Part ID 0x15" for a MAX30102.
    println!("Part ID: {}", part_id);
}
