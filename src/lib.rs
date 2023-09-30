#![no_std]
#![recursion_limit = "1024"]

pub mod eeprom25x;
pub mod storage;
pub mod status;
pub mod register;

pub use eeprom25x::*;
pub use status::*;
pub use storage::*;
pub use register::*;