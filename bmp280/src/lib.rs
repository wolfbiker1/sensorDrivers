pub mod pressure_temp {
    use atomic_float::AtomicF64;
    use bmp280::Bmp280Builder;
    use std::sync::atomic::Ordering;

    pub static PRESSURE: AtomicF64 = AtomicF64::new(0.0);
    pub static TEMP: AtomicF64 = AtomicF64::new(0.0);
    const INTERVAL: u64 = 15;
    pub fn get_pressure() -> f64 {
        PRESSURE.load(Ordering::SeqCst)
    }

    pub fn get_temp() -> f64 {
        TEMP.load(Ordering::SeqCst)
    }

    fn set_pressure(pressure: f64) {
        PRESSURE.store(pressure, Ordering::SeqCst);
    }

    fn set_temp(temp: f64) {
        TEMP.store(temp, Ordering::SeqCst);
    }

    pub fn main_worker() -> ! {
        let mut dev = Bmp280Builder::new()
            .path("/dev/i2c-1")
            .address(0x76)
            .ground_pressure(965.54)
            .build()
            .expect("Failed to build device");

        dev.zero().expect("Device failed to zero");

        loop {
            let pressure = (100.0 / 95.0 * 10.0 * dev.pressure_kpa().unwrap()) as f64;
            set_pressure(pressure);
            let temp = (dev.temperature_celsius().unwrap()) as f64;
            set_temp(temp);

            std::thread::sleep(std::time::Duration::from_secs(INTERVAL));
        }
    }
}
