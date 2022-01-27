use am2302::humidity;
use bh1750::brightness;
use bmp280_module::pressure_temp;
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::{thread, time};
const SOCKET_ADDR: &str = "192.168.178.66:7879";

#[derive(Serialize, Deserialize, Debug)]
struct Measurements {
    temperature: String,
    humidity: String,
    pressure: String,
    brightness: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Timestamps {
    // indoor_temp: String,
    // brightness: Instant,
    // pressure: Instant,
    outdoor_values: String,
}

fn send_to_backend(socket: &mut UdpSocket) {
    let measurements = Measurements {
        temperature: format!("{:.2}", humidity::get_outdoor_temp()),
        humidity: format!("{:.2}", humidity::get_humidity()),
        pressure: format!("{:.2}", pressure_temp::get_pressure()),
        brightness: format!("{:.2}", brightness::get_brightness()),
    };

    let timestamps = Timestamps {
        outdoor_values: humidity::get_timestamp().to_rfc3339(),
    };

    let measurements: String = serde_json::to_string(&measurements).unwrap();
    let timestamps: String = serde_json::to_string(&timestamps).unwrap();

    socket
        .send_to(&measurements.len().to_ne_bytes(), SOCKET_ADDR)
        .expect("couldn't send data");
    socket
        .send_to(measurements.as_bytes(), SOCKET_ADDR)
        .expect("couldn't send data");
    socket
        .send_to(&timestamps.len().to_ne_bytes(), SOCKET_ADDR)
        .expect("couldn't send data");
    socket
        .send_to(timestamps.as_bytes(), SOCKET_ADDR)
        .expect("couldn't send data");
}

fn main() {
    let t1 = thread::spawn(|| {
        humidity::main_worker();
    });
    thread::sleep(time::Duration::from_secs(2));
    let t2 = thread::spawn(|| {
        pressure_temp::main_worker();
    });
    thread::sleep(time::Duration::from_secs(2));
    let t3 = thread::spawn(|| {
        brightness::main_worker();
    });
    thread::sleep(time::Duration::from_secs(2));
    let t4 = thread::spawn(|| {
        let mut socket = UdpSocket::bind("0.0.0.0:7877").expect("couldn't bind to address");
        loop {
            send_to_backend(&mut socket);
            thread::sleep(time::Duration::from_secs(15));
        }
    });

    t1.join().unwrap_or_default();
    t2.join().unwrap_or_default();
    t3.join().unwrap_or_default();
    t4.join().unwrap_or_default();
}
