use cec_rs::{CecConnectionCfgBuilder, CecLogicalAddress, CecDeviceTypeVec, CecDeviceType};

pub fn turn_tv_on() {
    // cec-client -s -d 1 "as"
    let conn = CecConnectionCfgBuilder::default()
        .device_name("test".into())
        .device_types(CecDeviceTypeVec::new(CecDeviceType::Tv))
        .build().unwrap().open().unwrap();
    conn.send_power_on_devices(CecLogicalAddress::Tv).unwrap();
}
pub fn turn_tv_off() {
    // cec-client -s -d 1 "standby 0"
    let conn = CecConnectionCfgBuilder::default().build().unwrap().open().unwrap();
    conn.send_power_on_devices(CecLogicalAddress::Tv).unwrap();
}
