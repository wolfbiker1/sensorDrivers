#[macro_use]
extern crate lazy_static;
pub mod humidity {
    use atomic_float::AtomicF64;
    use chrono::prelude::*;
    use gpio_cdev::{Chip, Line, LineRequestFlags};
    use std::sync::atomic::Ordering;
    use std::sync::RwLock;
    use std::{thread, time};
    use bitvec::prelude::*;

    const INTERVAL: u64 = 5;
    pub static HUMIDITY: AtomicF64 = AtomicF64::new(0.0);
    pub static OUTDOOR_TEMP: AtomicF64 = AtomicF64::new(0.0);

    lazy_static! {
        pub static ref LAST_UPDATED: RwLock<DateTime<Local>> = RwLock::new(Local::now());
    }

    pub fn get_humidity() -> f64 {
        HUMIDITY.load(Ordering::SeqCst)
    }

    fn set_humidity(humidity: f64) {
        HUMIDITY.store(humidity, Ordering::SeqCst);
    }

    pub fn get_outdoor_temp() -> f64 {
        OUTDOOR_TEMP.load(Ordering::SeqCst)
    }

    fn set_outdoor_temp(temp: f64) {
        OUTDOOR_TEMP.store(temp, Ordering::SeqCst);
    }

    fn set_timestamp() {
        let mut timestamp = LAST_UPDATED.write().unwrap();
        *timestamp = Local::now();
    }

    pub fn get_timestamp() -> DateTime<Local> {
        *LAST_UPDATED.write().unwrap()
    }

    pub fn main_worker() -> ! {
        let line: gpio_cdev::Line = get_line(4);
        loop {
            do_measurement(&line);
            std::thread::sleep(std::time::Duration::from_secs(INTERVAL));
        }
    }

    fn do_measurement(line: &gpio_cdev::Line) {
        do_init(line);
        let measure_results: Vec<std::time::Duration> = start_reading(line);
        if !crc_check_n_send(&measure_results) {
            do_measurement(line)
        }
    }

    fn do_init(line: &Line) {
        let output = line
            .request(LineRequestFlags::OUTPUT, 1, "request-data")
            .unwrap();
        output.set_value(0).unwrap();
        thread::sleep(time::Duration::from_millis(2));
    }

    fn convert_durations_to_bit(e: &[time::Duration]) -> BitVec<bitvec::order::Msb0, u8> {
        let mut bit_vec: BitVec<bitvec::order::Msb0, u8> = bitvec![Msb0, u8;];
        for (n, elapsed) in e.iter().enumerate() {
            if n == 0 || n == 1 {
                continue
            }
            if elapsed.as_micros() >= 19 && elapsed.as_micros() <= 30 {
                bit_vec.push(false);
            } else if elapsed.as_micros() >= 68 && elapsed.as_micros() <= 82 {
                bit_vec.push(true);
            }
        }    
        bit_vec  
    }

    fn crc_check_n_send(e: &[time::Duration]) -> bool {
        let bit_vec: BitVec<bitvec::order::Msb0, u8> = convert_durations_to_bit(e);

        if bit_vec.len() != 40 {
            return false
        }

        let rh_high = bit_vec.as_raw_slice()[0] as usize;
        let rh_low = bit_vec.as_raw_slice()[1] as usize;
        let t_high = bit_vec.as_raw_slice()[2] as usize;
        let t_low = bit_vec.as_raw_slice()[3] as usize;
        let checksum =  bit_vec.as_raw_slice()[4] as usize;

        if (rh_high + rh_low + t_high + t_low) as u8 == checksum as u8 {
            set_timestamp();
            set_humidity(u16::from_be_bytes([rh_high as u8, rh_low as u8]) as f64 / 10.0);
            set_outdoor_temp(u16::from_be_bytes([t_high as u8, t_low as u8]) as f64 / 10.0);
            return true
        }
        false
    }

    fn start_reading(line: &Line) -> Vec<time::Duration> {
        let reading_time = time::Duration::from_millis(5);
        let mut elapsed_times: Vec<time::Duration> = Vec::new();
        let input = line
            .request(LineRequestFlags::INPUT, 1, "request-data")
            .unwrap();

        let mut previous_bit_state = input.get_value().unwrap();
        let start_time = time::Instant::now();
        let mut bit_start = time::Instant::now();

        while start_time.elapsed() < reading_time {
            let current_bit_state = input.get_value().unwrap();
            if current_bit_state != previous_bit_state {
                let bit_end = time::Instant::now();

                if previous_bit_state == 1 && current_bit_state == 0 {
                    elapsed_times.push(bit_end - bit_start);
                }
                bit_start = bit_end;
                previous_bit_state = current_bit_state;
            }
        }
        elapsed_times
    }

    fn get_line(gpio_number: u32) -> Line {
        let mut chip = Chip::new("/dev/gpiochip0").unwrap();
        chip.get_line(gpio_number).unwrap()
    }
}
