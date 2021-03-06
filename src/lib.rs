#![no_std]

// use imxrt_hal as hal;
use imxrt_ral as ral;

pub mod channels;
pub mod dcp;
pub mod ex;
pub mod ops;
pub mod packet;
pub mod task;

/// Derived from the DCP status field when an operation fails.
/// Holds the error kind and an 8 bit error code.
// I haven't been able to find a way to interpret the 8 bit error codes, if anyone finds something
// useful please submit a PR
#[derive(Debug)]
pub enum Error {
    HashMismatch(u8),
    SetupError(u8),
    PacketError(u8),
    SourceError(u8),
    DestError(u8),
    Other(u8)
}

pub type Result = nb::Result<u8, Error>;

pub mod prelude {
    pub use crate::{
        ex::Executor,
        ops::{self, config::*},
        packet::PacketBuilder,
    };
}
