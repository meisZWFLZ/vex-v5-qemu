#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use battery::BatteryData;
use bincode::{Decode, Encode};
use code_signature::CodeSignature;
use controller::{ControllerData, ControllerId};
use rotation_sensor::RotationSensorData;
use core::{option::Option, num::NonZeroU32};
use display::{Color, DisplayRenderMode, DrawCommand, ScrollLocation};
use distance_sensor::DistanceSensorData;
use imu::ImuData;
use geometry::Rect;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod battery;
pub mod code_signature;
pub mod controller;
pub mod display;
pub mod distance_sensor;
pub mod geometry;
pub mod imu;
pub mod motor;
pub mod rotation_sensor;

/// A message sent from the guest to the host.
#[derive(Debug, Clone, PartialEq, PartialOrd, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum HostBoundPacket {
    UsbSerial(Vec<u8>),
    KernelSerial(Vec<u8>),
    CodeSignature(CodeSignature),
    ExitRequest(i32),
    LinkAddressRequest,
    DisplayCommand { command: DisplayCommand },
    SmartPortCommand { port: u8, command: SmartPortCommand },
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DisplayCommand {
    Draw {
        command: DrawCommand,
        color: Color,
        clip_region: Rect,
    },
    Scroll {
        location: ScrollLocation,
        lines: i32,
        background: Color,
        clip_region: Rect,
    },
    Erase {
        color: Color,
        clip_region: Rect,
    },
    DisableDoubleBuffering,
    Render,
}

/// A message sent from the host to the guest.
#[derive(Debug, Clone, PartialEq, PartialOrd, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum KernelBoundPacket {
    UsbSerial(Vec<u8>),
    LinkAddress(Option<NonZeroU32>),
    SmartPortUpdate {
        port_index: u8,
        data: SmartPortData,
        timestamp: u32,
    },
    ControllerUpdate {
        id: ControllerId,
        data: ControllerData,
        timestamp: u32,
    },
    BatteryUpdate {
        data: BatteryData,
        timestamp: u32,
    },
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SmartPortData {
    DistanceSensor(DistanceSensorData),
    Imu(ImuData),
    RotationSensor(RotationSensorData)
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SmartPortCommand {}

#[macro_export]
macro_rules! impl_bincode_bitflags {
    ($flags:ty) => {
        impl bincode::enc::Encode for $flags {
            fn encode<E: ::bincode::enc::Encoder>(
                &self,
                encoder: &mut E,
            ) -> Result<(), bincode::error::EncodeError> {
                bincode::Encode::encode(&self.bits(), encoder)?;
                Ok(())
            }
        }

        impl bincode::de::Decode for $flags {
            fn decode<D: bincode::de::Decoder>(
                decoder: &mut D,
            ) -> Result<Self, bincode::error::DecodeError> {
                Ok(Self::from_bits_retain(bincode::Decode::decode(decoder)?))
            }
        }

        impl<'de> bincode::BorrowDecode<'de> for $flags {
            fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
                decoder: &mut D,
            ) -> Result<Self, bincode::error::DecodeError> {
                Ok(Self::from_bits_retain(
                    bincode::BorrowDecode::borrow_decode(decoder)?,
                ))
            }
        }
    };
}
