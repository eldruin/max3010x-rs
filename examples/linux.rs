use linux_embedded_hal::I2cdev;
use max3010x::{Led, Max3010x, SampleAveraging};

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let mut sensor = Max3010x::new_max30102(dev);
    let part_id = sensor.get_part_id().unwrap();

    // This should print "Part ID 0x15" for a MAX30102.
    println!("Part ID: {}", part_id);

    let mut sensor = sensor.into_heart_rate().unwrap();
    sensor.set_sample_averaging(SampleAveraging::Sa4).unwrap();
    sensor.set_pulse_amplitude(Led::All, 15).unwrap();
    sensor.enable_fifo_rollover().unwrap();
    let mut data = [0; 3];
    let samples_read = sensor.read_fifo(&mut data).unwrap();

    println!("Samples read: {}", samples_read);
}
