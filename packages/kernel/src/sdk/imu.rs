//! V5 Inertial Sensor

use core::{f64::{INFINITY, NAN}, ffi::c_double};

use vex_sdk::*;
use vex_v5_qemu_protocol::SmartPortData;

use super::SMARTPORTS;

pub extern "C" fn vexDeviceImuReset(device: V5_DeviceT) {}
// Used by PROS's imu_get_rotation. Should get the unbounded rotation
pub extern "C" fn vexDeviceImuHeadingGet(device: V5_DeviceT) -> c_double {
    if let Some(port) = SMARTPORTS.get(unsafe { *device }.zero_indexed_port as usize) {
        if let Some(SmartPortData::Imu(data)) = &port.lock().data {
            return if let Some(object) = &data.object {
                object.heading
            } else {
                NAN
            }
        }
    }
    // TODO: Check if this is a good error value
    INFINITY
}
// Used by PROS's imu_get_heading. Perhaps bounded to (-180, 180)?
pub extern "C" fn vexDeviceImuDegreesGet(device: V5_DeviceT) -> c_double {
    if let Some(port) = SMARTPORTS.get(unsafe { *device }.zero_indexed_port as usize) {
        if let Some(SmartPortData::Imu(data)) = &port.lock().data {
            return if let Some(object) = &data.object {
                object.degrees
            } else {
                NAN
            }
        }
    }
    // TODO: Check if this is a good error value
    INFINITY
}
pub extern "C" fn vexDeviceImuQuaternionGet(device: V5_DeviceT, data: *mut V5_DeviceImuQuaternion) {
}
pub extern "C" fn vexDeviceImuAttitudeGet(device: V5_DeviceT, data: *mut V5_DeviceImuAttitude) {}
pub extern "C" fn vexDeviceImuRawGyroGet(device: V5_DeviceT, data: *mut V5_DeviceImuRaw) {}
pub extern "C" fn vexDeviceImuRawAccelGet(device: V5_DeviceT, data: *mut V5_DeviceImuRaw) {}
pub extern "C" fn vexDeviceImuStatusGet(device: V5_DeviceT) -> u32 {
    if let Some(port) = SMARTPORTS.get(unsafe { *device }.zero_indexed_port as usize) {
        if let Some(SmartPortData::Imu(data)) = &port.lock().data {
            return data.status;
        }
    }

    0
}
pub extern "C" fn vexDeviceImuTemperatureGet(device: V5_DeviceT) -> c_double {
    if let Some(port) = SMARTPORTS.get(unsafe { *device }.zero_indexed_port as usize) {
        if let Some(SmartPortData::Imu(data)) = &port.lock().data {
            return if let Some(object) = &data.object {
                object.temperature
            } else {
                NAN
            }
        }
    }
    0.
}
pub extern "C" fn vexDeviceImuModeSet(device: V5_DeviceT, mode: u32) {}
pub extern "C" fn vexDeviceImuModeGet(device: V5_DeviceT) -> u32 {
    if let Some(port) = SMARTPORTS.get(unsafe { *device }.zero_indexed_port as usize) {
        if let Some(SmartPortData::Imu(data)) = &port.lock().data {
            return if let Some(object) = &data.object {
                object.mode
            } else {
                9999
            }
        }
    }
    // TODO: Check if this is a good error value
    0
}
pub extern "C" fn vexDeviceImuDataRateSet(device: V5_DeviceT, rate: u32) {}
