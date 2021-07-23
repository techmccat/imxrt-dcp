use super::ral::{self, dcp::{RegisterBlock, CHANNELCTRL::ENABLE_CHANNEL::RW as ch}, write_reg};
use crate::packet::raw::ControlPacket;

/// Trait implemented for DCP concurrent channels.
pub trait Channel {
    const CHANNEL_BIT: u32;

    /// Schedules the execution of a packet in the channel.
    fn write_cmdptr(inst: &RegisterBlock, ptr: &ControlPacket);
    /// Starts the pending operation(s).
    fn incr_semaphore(inst: &RegisterBlock, value: u32);
    /// Clears the status register of the channel. Called at the end of an operation.
    fn clear_status(inst: &RegisterBlock);

    /// Enables the channel. Enabling more than one channel requires setting up a context switch
    /// buffer for the DCP.
    fn enable(inst: &RegisterBlock) {
        write_reg!(ral::dcp, inst, CHANNELCTRL_SET, Self::CHANNEL_BIT);
    }
}

pub struct Ch0;
pub struct Ch1;
pub struct Ch2;
pub struct Ch3;

macro_rules! write_cmdptr {
    ( $reg:ident ) => {
        fn write_cmdptr(inst: &RegisterBlock, ptr: &ControlPacket) {
            let raw_ptr = ptr as *const ControlPacket as u32;
            write_reg!(ral::dcp, inst, $reg, raw_ptr)
        }
    };
}

macro_rules! incr_semaphore {
    ( $reg:ident ) => {
        fn incr_semaphore(inst: &RegisterBlock, value: u32) {
            write_reg!(ral::dcp, inst, $reg, value)
        }
    };
}

macro_rules! clear_status {
    ( $reg:ident ) => {
        fn clear_status(inst: &RegisterBlock) {
            write_reg!(ral::dcp, inst, $reg, u32::MAX)
        }
    };
}

impl Channel for Ch0 {
    const CHANNEL_BIT: u32 = ch::CH0;
    write_cmdptr!(CH0CMDPTR);
    incr_semaphore!(CH0SEMA);
    clear_status!(CH0STAT_CLR);
}

impl Channel for Ch1 {
    const CHANNEL_BIT: u32 = ch::CH1;
    write_cmdptr!(CH1CMDPTR);
    incr_semaphore!(CH1SEMA);
    clear_status!(CH1STAT_CLR);
}

impl Channel for Ch2 {
    write_cmdptr!(CH2CMDPTR);
    const CHANNEL_BIT: u32 = ch::CH2;
    incr_semaphore!(CH2SEMA);
    clear_status!(CH2STAT_CLR);
}

impl Channel for Ch3 {
    const CHANNEL_BIT: u32 = ch::CH3;
    write_cmdptr!(CH3CMDPTR);
    incr_semaphore!(CH3SEMA);
    clear_status!(CH3STAT_CLR);
}
