use super::ral::{
    self,
    dcp::{RegisterBlock, CHANNELCTRL::ENABLE_CHANNEL::RW as ch},
    read_reg, write_reg,
};
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
    /// Checks if the channel is in use.
    fn busy(inst: &RegisterBlock) -> bool;

    /// Enables the channel and clears its status.
    fn enable(inst: &RegisterBlock) {
        write_reg!(ral::dcp, inst, CHANNELCTRL_SET, Self::CHANNEL_BIT);
        Self::clear_status(inst);
    }

    /// Disables the channel and clears its status.
    fn disable(inst: &RegisterBlock) {
        Self::clear_status(inst);
        write_reg!(ral::dcp, inst, CHANNELCTRL_CLR, Self::CHANNEL_BIT);
    }

    /// Clears the status and writes a control packet pointer.
    fn clear_and_cmdptr(inst: &RegisterBlock, ptr: &ControlPacket) {
        Self::clear_status(inst);
        Self::write_cmdptr(inst, ptr);
    }
}

pub struct Ch<const N: u8>;

pub type Ch0 = Ch<0>;
pub type Ch1 = Ch<1>;
pub type Ch2 = Ch<2>;
pub type Ch3 = Ch<3>;

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

macro_rules! busy {
    ( $reg:ident ) => {
        fn busy(inst: &RegisterBlock) -> bool {
            read_reg!(ral::dcp, inst, $reg, VALUE != 0)
        }
    };
}

impl Channel for Ch<0> {
    const CHANNEL_BIT: u32 = ch::CH0;
    write_cmdptr!(CH0CMDPTR);
    incr_semaphore!(CH0SEMA);
    clear_status!(CH0STAT_CLR);
    busy!(CH0SEMA);
}

impl Channel for Ch<1> {
    const CHANNEL_BIT: u32 = ch::CH1;
    write_cmdptr!(CH1CMDPTR);
    incr_semaphore!(CH1SEMA);
    clear_status!(CH1STAT_CLR);
    busy!(CH1SEMA);
}

impl Channel for Ch<2> {
    write_cmdptr!(CH2CMDPTR);
    const CHANNEL_BIT: u32 = ch::CH2;
    incr_semaphore!(CH2SEMA);
    clear_status!(CH2STAT_CLR);
    busy!(CH2SEMA);
}

impl Channel for Ch<3> {
    const CHANNEL_BIT: u32 = ch::CH3;
    write_cmdptr!(CH3CMDPTR);
    incr_semaphore!(CH3SEMA);
    clear_status!(CH3STAT_CLR);
    busy!(CH3SEMA);
}
