//! Execute DCP work packets.
//!
//! DCP packets need to be passed to the hardware to be ran.
//! Executors handle that.

use core::marker::PhantomData;
use imxrt_ral::{dcp, write_reg};

use crate::{channels::*, dcp::DCP, packet::raw::ControlPacket};

/// Errors encountered while queueing a task for execution.
#[derive(Debug)]
pub enum ExError {
    /// There is no
    SlotsFull,
}

/// Executes [`Task`](crate::task::Task)s
pub trait Executor: Sized {
    /// Executes a single task.
    ///
    /// Returns [`SlotsFull`](ExError::SlotsFull) if the queue (if there is any) is full.
    fn exec_one(&self, task: &mut ControlPacket) -> Result<(), ExError> {
        unsafe { self.inner_exec(task) }
    }

    /// Same as `exec_one`, but executes a contiguous slice of `[Task]`s.
    fn exec_slice(&self, tasks: &mut [ControlPacket]) -> Result<(), ExError> {
        if let Some((_, most)) = tasks.split_last_mut() {
            for task in most {
                task.control0.chain_continuous()
            }
            unsafe { self.inner_exec(&mut tasks[0]) }
        } else {
            Ok(())
        }
    }

    /// Implementation-specific function called by the other methods.
    ///
    /// # Unsafe
    ///
    /// Implementor must guarantee that the `[Task]` is not moved after execution.
    unsafe fn inner_exec(&self, task: &mut ControlPacket) -> Result<(), ExError>;
}

/// A single channel [`Executor`] that does not need a context switch buffer.
pub struct SingleChannel<C: Channel> {
    inst: DCP,
    _chan: PhantomData<C>,
}

impl<C: Channel> SingleChannel<C> {
    /// Builds `Self` from a `[Builder]`
    pub fn new(inst: DCP) -> Self {
        C::clear_status(&inst);
        C::enable(&inst);

        Self {
            inst,
            _chan: PhantomData
        }
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
            task.control0.decr_semaphore();
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

        write_reg!(dcp, &inst, CTRL_SET, dcp::CTRL::ENABLE_CONTEXT_SWITCHING::mask);
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
