use cec_rs::{CecConnectionCfgBuilder, CecLogicalAddress, CecDeviceTypeVec, CecDeviceType};
use std::process::Command;
use std::io;
use std::io::Write;

pub fn turn_tv_on() {
    // active source
    send_command(&["-s", "-t0", "--active-source", "phys-addr=2.0.0.0"]);
}
pub fn turn_tv_off() {
    // put address 0 in standby
    send_command(&["-s", "-t0", "--standby"]);
}

pub fn send_command(args: &[&str]) {
    let output = Command::new("cec-ctl")
        .args(args)
        .output()
        .expect("failed to start cec-ctl");

    io::stdout().write_all(&output.stdout).unwrap();
}
