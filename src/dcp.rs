//! Code for managing the DCP hardware.
//!
//! This module contains structs and functions to clock and unclock the peripheral and manage its
//! state.
use core::ops::Deref;

use imxrt_ral as ral;
use ral::{dcp, modify_reg, write_reg};

/// Unclocked DCP instance.
///
/// This needs to be clocked before it can be used.
pub struct Unclocked {
    inst: dcp::Instance,
}

impl Unclocked {
    /// Creates a new `Unclocked` by taking the DCP instance.
    pub fn take() -> Option<Self> {
        dcp::DCP::take().map(|inst| Self { inst })
    }

    /// Turn on clocking
    pub fn clock(self, ccm: &ral::ccm::Instance) -> Builder {
        // Turn the DCP clock on
        modify_reg!(ral::ccm, ccm, CCGR0, |r| r | ral::ccm::CCGR0::CG5::mask);

        Builder { inst: self.inst }
    }

    /// Releases the DCP instance.
    pub fn release(self) {
        dcp::DCP::release(self.inst)
    }

    /// Returns a reference to the instance.
    pub fn raw(&self) -> &dcp::Instance {
        &self.inst
    }
}

pub struct Builder {
    pub(crate) inst: dcp::Instance,
}

/// Set DCP configuration before enabling it. (TBD)
///
/// In this state the peripheral is clocked but not enabled.
impl Builder {
    /// Enable the DCP.
    /// 
    /// This function resets the DCP, enables it and sets some necessary register flags.
    pub fn build(self) -> DCP {
        // Set CLKGATE to zero
        write_reg!(dcp, self.inst, CTRL_CLR, ral::dcp::CTRL::CLKGATE::mask);
        // Reset the DCP to the default state
        // Set the SFTRST bit in the control register high
        write_reg!(dcp, self.inst, CTRL_SET, ral::dcp::CTRL::SFTRST::mask);
        // Then set it low to enable operation
        write_reg!(dcp, self.inst, CTRL_CLR, ral::dcp::CTRL::SFTRST::mask);
        // Clear DCP status
        // Sets the first 4 bits from the STAT register to 0, clearing pending interrupts
        write_reg!(dcp, self.inst, STAT_CLR, ral::dcp::STAT::IRQ::mask);

        // Enable residual writes for faster unaligned operations
        let ctrl_reg = ral::dcp::CTRL::GATHER_RESIDUAL_WRITES::mask
        // Context caching
        | ral::dcp::CTRL::ENABLE_CONTEXT_CACHING::mask;
        write_reg!(dcp, self.inst, CTRL_SET, ctrl_reg);

        DCP(self.inst)
    }
}

/// Clocked and active DCP peripheral.
pub struct DCP(pub/*(crate)*/ dcp::Instance);

impl DCP {
    /// Resets the DCP and disables clock.
    pub fn unclock(self, ccm: &ral::ccm::Instance) -> Unclocked {
        let inst = self.0;
        // Clear interrupts
        write_reg!(dcp, inst, STAT_CLR, ral::dcp::STAT::IRQ::mask);
        // Put the DCP in its reset state
        write_reg!(dcp, inst, CTRL_SET, ral::dcp::CTRL_SET::SFTRST::mask);
        // Turn the DCP clock off
        modify_reg!(ral::ccm, ccm, CCGR0, |r| r ^ ral::ccm::CCGR0::CG5::mask);

        Unclocked { inst }
    }
}

impl Deref for DCP {
    type Target = dcp::Instance;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
