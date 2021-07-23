#![no_std]

// use imxrt_hal as hal;
use imxrt_ral as ral;

// use hal::ccm;
use ral::{modify_reg, write_reg, dcp};

pub mod channels;
pub mod ops;
pub mod packet;
//pub mod task;

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

/// Enable the DCP and one channel and return an instance to it.
pub fn setup<C: channels::Channel>(ccm_regs: &mut ral::ccm::RegisterBlock) -> ral::dcp::Instance {
    // Take CCM handle and return an Instance (derefs as RegisterBlock)
    // Take RegisterBlock for the DCP
    let dcp_regs = ral::dcp::DCP::take().unwrap();

    // Turn the DCP on
    modify_reg!(ral::ccm, ccm_regs, CCGR0, |reg| reg + ral::ccm::CCGR0::CG5::mask);

    // Reset the DCP to the default state
    // Set the SFTRST bit in the control register high
    write_reg!(dcp,
        dcp_regs,
        CTRL_CLR,
        ral::dcp::CTRL_SET::SFTRST::mask );
    // Then set it low to enable operation
    write_reg!(dcp,
        dcp_regs,
        CTRL_SET,
        ral::dcp::CTRL_SET::SFTRST::mask );
    // Enable residual writes for faster unaligned operations
    let ctrl_reg = ral::dcp::CTRL::GATHER_RESIDUAL_WRITES::mask
    // Context caching
        | ral::dcp::CTRL::ENABLE_CONTEXT_CACHING::mask;
    write_reg!(dcp, dcp_regs, CTRL_SET, ctrl_reg);

    // Clear DCP status
    // Sets the first 4 bits from the STAT register to 0, clearing pending interrupts
    write_reg!(dcp, dcp_regs, STAT_CLR, ral::dcp::STAT::IRQ::mask);

    // Clear channel status
    C::clear_status(&dcp_regs);
    // TODO: find a way to expose multiple channels
    // Maybe with async stuff?
    // Interrupts are a PITA and polling is probably the way to go about this.
    // Never wrote stuff with futures, will eventually™ check them out

    // Enable channels
    // NOTE: enabling more channels requires a context switch buffer, 208 bytes of wasted
    // RAM if not used
    C::enable(&dcp_regs);

    dcp_regs
}

pub mod prelude {
    pub use crate::{
        setup,
        ops::{self, config::*},
        packet::PacketBuilder
    };
}
