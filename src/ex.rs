//! Execute DCP work packets.
//!
//! DCP packets need to be passed to the hardware to be ran.
//! Executors handle that.

use core::marker::PhantomData;
use imxrt_ral::{dcp, write_reg};

use crate::{
    channels::*,
    dcp::DCP,
    packet::{Control0Flag, ControlPacket},
};

/// Errors encountered while queueing a task for execution.
#[derive(Debug)]
pub enum ExError {
    /// All the channels are full
    SlotsFull,
}

/// Executes [`Task`]s
pub trait Executor {
    /// Executes a single task.
    ///
    /// Returns [`SlotsFull`](ExError::SlotsFull) if the queue (if there is any) is full.
    fn exec_one<'a>(&self, task: &'a mut ControlPacket<'a>) -> Result<Task<'a>, ExError> {
        unsafe { self.inner_exec(task) }?;
        Ok(Task { packet: task })
    }

    /// Same as `exec_one`, but executes a contiguous slice of `Task`s.
    ///
    /// Panics if slice is empty.
    fn exec_slice<'a>(&self, tasks: &'a mut [ControlPacket<'a>]) -> Result<Task<'a>, ExError> {
        let (_, most) = tasks.split_last_mut().unwrap();
        for task in most {
            task.control0 = task.control0.flag(Control0Flag::ChainContinuous)
        }
        unsafe { self.inner_exec(&mut tasks[0]) }?;
        Ok(Task { packet: tasks.last_mut().unwrap() })
    }

    /// Implementation-specific function called by the other methods.
    ///
    /// # Unsafe
    ///
    /// Implementor must guarantee that the ControlPacket is not moved after execution.
    unsafe fn inner_exec(&self, task: &mut ControlPacket) -> Result<(), ExError>;
}

/// A single channel [`Executor`] that does not need a context switch buffer.
pub struct SingleChannel<C: Channel> {
    pub inst: DCP,
    _chan: PhantomData<C>,
}

impl<C: Channel> SingleChannel<C> {
    pub fn take(inst: DCP) -> Option<Self> {
        if C::enabled(&inst) {
            return None;
        }
        C::clear_status(&inst);
        C::enable(&inst);

        Some(Self {
            inst,
            _chan: PhantomData,
        })
    }

    /// Blocks until tasks are complete and returns a `[Builder]`.
    pub fn release(self) -> DCP {
        // block until the channel is free
        while C::busy(&self.inst) {}

        C::disable(&self.inst);

        self.inst
    }
}

impl<C: Channel> Executor for SingleChannel<C> {
    unsafe fn inner_exec(&self, task: &mut ControlPacket) -> Result<(), ExError> {
        if C::busy(&self.inst) {
            Err(ExError::SlotsFull)
        } else {
            task.control0.flag(Control0Flag::DecrSemaphore);
            C::clear_and_cmdptr(&self.inst, task);
            C::incr_semaphore(&self.inst, 1);

            Ok(())
        }
    }
}

/// A scheduler that manages multiple channels.
pub struct Scheduler<'a> {
    inst: DCP,
    _ctx: &'a mut [u8; 208],
}

impl<'a> Scheduler<'a> {
    /// Takes a memory region for the context switching buffer and returns a scheduler.
    ///
    /// If you don't want to worry about lifetimes i recommend allocating a static buffer and
    /// being done with it.
    pub fn new(inst: DCP, buf: &'a mut [u8; 208]) -> Self {
        Ch0::enable(&inst);
        Ch1::enable(&inst);
        Ch2::enable(&inst);
        Ch3::enable(&inst);

        write_reg!(
            dcp,
            &inst,
            CTRL_SET,
            dcp::CTRL::ENABLE_CONTEXT_SWITCHING::mask
        );
        write_reg!(dcp, &inst, CONTEXT, buf as *const u8 as u32);

        Self { inst, _ctx: buf }
    }

    /// Checks if there are channels with nonzero semaphore.
    pub fn busy(&self) -> bool {
        Ch0::busy(&self.inst)
            || Ch1::busy(&self.inst)
            || Ch2::busy(&self.inst)
            || Ch3::busy(&self.inst)
    }

    /// Blocks until all channels have completed, disables the channels and returns the DCP instance.
    pub fn release(self) -> DCP {
        while self.busy() {}

        Ch0::disable(&self.inst);
        Ch1::disable(&self.inst);
        Ch2::disable(&self.inst);
        Ch3::disable(&self.inst);

        self.inst
    }
}

impl<'a> Executor for Scheduler<'a> {
    unsafe fn inner_exec(&self, task: &mut ControlPacket) -> Result<(), ExError> {
        if !Ch3::busy(&self.inst) {
            Ch3::clear_and_cmdptr(&self.inst, task);
            Ch3::incr_semaphore(&self.inst, 1);
        } else if !Ch2::busy(&self.inst) {
            Ch2::clear_and_cmdptr(&self.inst, task);
            Ch2::incr_semaphore(&self.inst, 1);
        } else if !Ch1::busy(&self.inst) {
            Ch1::clear_and_cmdptr(&self.inst, task);
            Ch1::incr_semaphore(&self.inst, 1);
        } else if !Ch0::busy(&self.inst) {
            Ch0::clear_and_cmdptr(&self.inst, task);
            Ch0::incr_semaphore(&self.inst, 1);
        } else {
            return Err(ExError::SlotsFull);
        }
        Ok(())
    }
}

/// Task object to poll for completion
///
/// The [Drop] implementation on this waits for completion of the operation and then discards the
/// result to prevent the DCP from holding a dangling pointers to the work packet and the buffers.
pub struct Task<'a> {
    packet: &'a mut ControlPacket<'a>,
}

impl Task<'_> {
    pub fn poll(&self) -> crate::Result {
        self.packet.status.poll()
    }
}

impl Drop for Task<'_> {
    fn drop(&mut self) {
        let _ = nb::block!(self.poll());
    }
}
