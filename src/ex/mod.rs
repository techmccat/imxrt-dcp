use core::{marker::PhantomData, pin::Pin};
use imxrt_ral::{dcp, write_reg};

use crate::{channels::*, dcp::Builder, task::Task};

/// Error returned when the `[Executor]` does not have space left to enqueue the task
pub struct SlotsFull;

/// Executes `[Task]`s
pub trait Executor {
    /// Executes a single task.
    ///
    /// Returns [SlotsFull] if the queue (if there is any) is full.
    fn exec_one(&self, task: Pin<&mut Task>) -> Result<(), SlotsFull> {
        // I feel like getting a pin and calling get_unchecked_mut is kind of pointless, but i'm
        // not sure I understand this pinning stuff yet.
        unsafe { self.inner_exec(task.get_unchecked_mut()) }
    }

    /// Same as `exec_one`, but executes a contiguous slice of `[Task]`s.
    fn exec_slice(&self, tasks: Pin<&mut [Task]>) -> Result<(), SlotsFull> {
        let slice_mut = unsafe { tasks.get_unchecked_mut() };
        if let Some((last, most)) = slice_mut.split_last_mut() {
            for task in most {
                task.control0.chain_continuous()
            }
            unsafe { self.inner_exec(last) }
        } else {
            Ok(())
        }
    }

    /// Implementation-specific function called by the other methods.
    ///
    /// # Unsafe
    ///
    /// Implementor must guarantee that the `[Task]` is not moved after execution.
    unsafe fn inner_exec(&self, task: &mut Task) -> Result<(), SlotsFull>;
}

/// A single channel `[Executor]` that does not need a context switch buffer.
pub struct SingleChannel<C: Channel> {
    inst: dcp::Instance,
    _chan: PhantomData<C>,
}

impl<C: Channel> SingleChannel<C> {
    /// Builds `Self` from a `[Builder]`
    pub fn new(builder: Builder) -> Self {
        builder.setup();
        let inst = builder.inst;

        C::clear_status(&inst);
        C::enable(&inst);

        Self {
            inst,
            _chan: PhantomData
        }
    }

    /// Blocks until tasks are complete and returns a `[Builder]`.
    pub fn release(self) -> Builder {
        // block until the channel is free
        while C::busy(&self.inst) {}

        C::disable(&self.inst);

        Builder {
            inst: self.inst
        }
    }
}

impl<C: Channel> Executor for SingleChannel<C> {
    unsafe fn inner_exec(&self, task: &mut Task) -> Result<(), SlotsFull> {
        if C::busy(&self.inst) {
            Err(SlotsFull)
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
    inst: dcp::Instance,
    _ctx: &'a mut [u8; 208],
}

impl<'a> Scheduler<'a> {
    pub fn new(builder: Builder, buf: &'a mut [u8; 208]) -> Self {
        builder.setup();
        let inst = builder.inst;

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
    pub fn release(self) -> Builder {
        while self.busy() {}

        Ch0::disable(&self.inst);
        Ch1::disable(&self.inst);
        Ch2::disable(&self.inst);
        Ch3::disable(&self.inst);

        Builder {
            inst: self.inst
        }
    }
}

impl<'a> Executor for Scheduler<'a> {
    unsafe fn inner_exec(&self, task: &mut Task) -> Result<(), SlotsFull> {
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
            return Err(SlotsFull);
        }
        Ok(())
    }
}
