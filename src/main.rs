mod bank;
mod color;
mod envelope;
mod envelope_gen;
mod event;
mod fixture;
mod organ;
mod patch;
mod store;
mod ui;

use std::{thread::sleep, time::Duration};

use rust_dmx::available_ports;

fn main() {
    let app = ui::TemplateApp::default();
    eframe::run_native(Box::new(app));
}
