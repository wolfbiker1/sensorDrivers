pub mod brightness {
    use atomic_float::AtomicF64;
    use i2cdev::core::*;
    use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
    use std::sync::atomic::Ordering;

    const INTERVAL: u64 = 15;
    const POWERON: &[u8] = &[0b0000_0001];
    const RESET: &[u8] = &[0b0000_0111];
    const MEASURE_CONTINOUS_H_RES: &[u8] = &[0b0010_0000];

    pub static BRIGHTNESS: AtomicF64 = AtomicF64::new(0.0);

    pub fn get_brightness() -> f64 {
        BRIGHTNESS.load(Ordering::SeqCst)
    }

    fn set_brightness(brightness: f64) {
        BRIGHTNESS.store(brightness, Ordering::SeqCst);
    }

    fn write_to_dev(device: &mut LinuxI2CDevice, data: &[u8]) {
        device.write(data).unwrap_or_default();
    }
    fn read_from_dev(device: &mut LinuxI2CDevice) -> Result<[u8; 2], LinuxI2CError> {
        let mut buffer: [u8; 2] = [0, 0];
        device.read(&mut buffer).unwrap_or_default();
        Ok(buffer)
    }

    fn convert_to_lux(data: &[u8]) -> f64 {
        ((data[0] as u32) << 8) as f64 + (data[1] as u32) as f64
    }

    pub fn main_worker() -> ! {
        let mut dev = LinuxI2CDevice::new("/dev/i2c-1", 0x23).unwrap();
        write_to_dev(&mut dev, POWERON);
        write_to_dev(&mut dev, RESET);
        loop {
            write_to_dev(&mut dev, MEASURE_CONTINOUS_H_RES);
            std::thread::sleep(std::time::Duration::from_millis(160));
            let result = read_from_dev(&mut dev);
            let lux_value = convert_to_lux(&result.unwrap());
            set_brightness(lux_value);
            std::thread::sleep(std::time::Duration::from_secs(INTERVAL));
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_conversion_1() {
            assert_eq!(convert_to_lux(&[0, 0]), 0.0);
        }
        #[test]
        fn test_conversion_2() {
            assert_eq!(convert_to_lux(&[8, 0]), 2048.0);
        }
        #[test]
        fn test_conversion_3() {
            assert_eq!(convert_to_lux(&[0x1F, 0x64]), 8036.0);
        }
    }
}
