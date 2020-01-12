use clap::{App, Arg, SubCommand};
use ctrlc;
use rplidar_drv::{Health, RplidarDevice, RplidarHostProtocol};
use rpos_drv::Channel;
use serialport::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{info, warn};
use pretty_env_logger::init;

fn main() {
    init();

    let s = &SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1),
    };
    let mut port = serialport::open_with_settings("/dev/ttyUSB0", &s).unwrap();

    port.write_data_terminal_ready(false).unwrap();

    let channel = Channel::<RplidarHostProtocol, dyn serialport::SerialPort>::new(
        RplidarHostProtocol::new(),
        port,
    );

    let mut device = RplidarDevice::new(channel);

    info!("{:?}", device.get_device_health().unwrap());

    device.start_motor().unwrap();
    device.start_scan().unwrap();

    let running = Arc::new(Mutex::from(device));
    let r = running.clone();

    let ended = Arc::new(AtomicBool::from(false));
    let en = ended.clone();

    ctrlc::set_handler(move || {
        en.store(true, Ordering::SeqCst);

        let mut rp_lock = r.lock().unwrap();
        rp_lock.stop_motor().unwrap();
        rp_lock.stop().unwrap();
    })
    .unwrap(); // TODO Doesn't work...

    while !ended.load(Ordering::SeqCst) {
        let mut rp = running.lock().unwrap();
        let scan_frame = rp.grab_scan_point_with_timeout(Duration::from_millis(5));
        println!("{:?}", scan_frame);
    }

    info!("Exiting...");
}
