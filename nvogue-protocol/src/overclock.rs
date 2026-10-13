use crate::units::{MHz, Watt};
use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Hash, Debug)]
pub struct OverclockInfo {
    // Graphics clock
    pub graphics_clock: ClockInfo,

    // Memory clock
    pub memory_clock: ClockInfo,

    // Power limit
    pub power_limit: PowerLimitInfo,
}

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Hash, Debug, Clone)]
pub struct ClockInfo {
    pub clock_type: ClockType,
    pub min_clock_offset: MHz,
    pub max_clock_offset: MHz,
    pub current_offset: MHz,
}

impl ClockInfo {
    pub fn new_empty(clock_type: ClockType) -> Self {
        ClockInfo {
            clock_type,
            min_clock_offset: MHz(0),
            max_clock_offset: MHz(0),
            current_offset: MHz(0),
        }
    }
}

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Hash, Debug, Copy, Clone)]
pub enum ClockType {
    Graphics,
    Memory,
}

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Hash, Debug)]
pub struct PowerLimitInfo {
    pub min_limit: Watt,
    pub max_limit: Watt,
    pub default_limit: Watt,
    pub current_limit: Watt,
}
