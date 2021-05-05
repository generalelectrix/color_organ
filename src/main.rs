mod bank;
mod color;
mod envelope;
mod envelope_gen;
mod event;
mod fixture;
mod organ;
mod patch;
mod store;

use std::{thread::sleep, time::Duration};

use rust_dmx::available_ports;

fn main() {
    for (name, open) in available_ports() {
        println!("{}", name);
        let mut port = open().expect(&format!("failed to open port {}", name));
        println!("opened {}", port);
        port.write(&vec![0; 512]).unwrap();
        sleep(Duration::from_secs(1));
        port.write(&vec![0; 512]).unwrap();
    }
}
