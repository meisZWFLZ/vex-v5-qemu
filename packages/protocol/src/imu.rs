use bincode::{Decode, Encode};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ImuData {
    pub object: Option<ImuObject>,
    pub status: u32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ImuObject {
    pub temperature: f64,
    pub heading: f64,
    pub degrees: f64,
    pub mode: u32
}
