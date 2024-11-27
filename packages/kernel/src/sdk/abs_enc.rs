//! V5 Rotation Sensor

use core::ffi::c_double;

use vex_sdk::*;
use vex_v5_qemu_protocol::SmartPortData;

use super::SMARTPORTS;

pub extern "C" fn vexDeviceAbsEncReset(device: V5_DeviceT) {}

pub extern "C" fn vexDeviceAbsEncPositionSet(device: V5_DeviceT, position: i32) {}

pub extern "C" fn vexDeviceAbsEncPositionGet(device: V5_DeviceT) -> i32 {
    if let Some(port) = SMARTPORTS.get(unsafe { *device }.zero_indexed_port as usize) {
        if let Some(SmartPortData::RotationSensor(data)) = &port.lock().data {
            return if let Some(object) = &data.object {
                object.position
            } else {
                9999
            }
        }
    }
    // TODO: Check if this is a good error value
    0
}

pub extern "C" fn vexDeviceAbsEncVelocityGet(device: V5_DeviceT) -> i32 {
    Default::default()
}

pub extern "C" fn vexDeviceAbsEncAngleGet(device: V5_DeviceT) -> i32 {
    Default::default()
}

pub extern "C" fn vexDeviceAbsEncReverseFlagSet(device: V5_DeviceT, value: bool) {
    Default::default()
}

pub extern "C" fn vexDeviceAbsEncReverseFlagGet(device: V5_DeviceT) -> bool {
    Default::default()
}

pub extern "C" fn vexDeviceAbsEncStatusGet(device: V5_DeviceT) -> u32 {
    if let Some(port) = SMARTPORTS.get(unsafe { *device }.zero_indexed_port as usize) {
        if let Some(SmartPortData::RotationSensor(data)) = &port.lock().data {
            return data.status;
        }
    }

    0
}

pub extern "C" fn vexDeviceAbsEncTemperatureGet(device: V5_DeviceT) -> c_double {
    Default::default()
}

pub extern "C" fn vexDeviceAbsEncDataRateSet(device: V5_DeviceT, rate: u32) {}
