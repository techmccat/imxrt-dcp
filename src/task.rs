use core::{
    marker::{PhantomData, PhantomPinned},
    ops::{Deref, DerefMut},
    pin::Pin,
};

use crate::{
    ex::Executor,
    ops::{config::*, *},
    packet::raw::{BlitSize, BufSize, Control1, ControlPacket},
};

// TODO: set buffers when constructing
/// A [`Task`] whose buffers and tag have not been set yet.
///
/// Once the buffers (and optionally the tag) have been set it can be frozen into an executable
/// [`Task`].
pub struct BlankTask<'a, O> {
    pub(crate) raw: ControlPacket<'a>,
    pub(crate) _op: PhantomData<O>,
}

impl<'a, O> BlankTask<'a, O> {
    pub fn set_tag(&mut self, tag: u8) {
        self.raw.status.tag = tag;
    }

    pub fn freeze<E: Executor>(self, ex: &'a mut E) -> Task<'a, E> {
        Task {
            ex,
            state: TaskState::Idle,
            inner: self.raw,
            _unpin: PhantomPinned,
        }
    }
}

impl<'a> BlankTask<'a, Memcopy> {
    /// Set source and destination buffers.
    ///
    /// # Panic
    ///
    /// This function panics if the payload buffer is not large enough.
    pub fn set_buffers(&mut self, src: CopySource<'a>, dst: &'a mut [u8]) {
        if let CopySource::ConstantFill(_) = src {
            self.raw.control0.constant_fill();
        }

        self.raw.source = src.into();
        self.raw.dest = dst.as_mut_ptr();
        self.raw.bufsize = BufSize { bufsize: dst.len() }
    }
}

impl<'a> BlankTask<'a, Blit> {
    /// Set source and destination buffers.
    ///
    /// # Panic
    ///
    /// This function panics if the payload buffer is not large enough.
    pub fn set_buffers(&mut self, src: CopySource<'a>, dst: Framebuffer<'a>) {
        if let CopySource::ConstantFill(_) = src {
            self.raw.control0.constant_fill();
        }

        self.raw.control1 = Control1::blit(dst.width);
        self.raw.source = src.into();
        self.raw.dest = dst.pointer.as_mut_ptr();
        self.raw.bufsize = BufSize {
            blit: BlitSize {
                width: dst.width,
                height: dst.height(),
            },
        };
    }
}

// TODO: Alignment optimizations, might want to look into https://crates.io/crates/aligned
impl<'a, C: CipherSelect> BlankTask<'a, Cipher<C>> {
    /// Set source, destination and payload buffers.
    ///
    /// # Panic
    ///
    /// This function panics if the payload buffer is not large enough.
    pub fn set_buffers(&mut self, mem: CryptMem<'a>, payl: &'a mut [u8]) {
        assert!(payl.len() >= C::PAYLOAD_BYTES);

        let len = match &mem {
            CryptMem::SourceDest(s, d) => {
                // There might be a better way than panicking, like returning a result
                assert_eq!(s.len(), d.len());
                s.len()
            }
            CryptMem::InPlace(sd) => sd.len(),
        };

        let (source, dest) = mem.into();

        self.raw.source = source;
        self.raw.dest = dest;
        self.raw.bufsize = BufSize { bufsize: len };
        self.raw.payload = payl.as_mut_ptr()
    }
}

impl<'a, H: HashSelect> BlankTask<'a, Hash<H>> {
    /// Set source and payload buffers.
    ///
    /// # Panic
    ///
    /// This function panics if the payload buffer is not large enough.
    pub fn set_buffers(&mut self, src: &'a [u8], payl: &'a mut [u8]) {
        assert!(payl.len() >= H::PAYLOAD_BYTES);

        self.raw.source = src.into();
        self.raw.bufsize = BufSize { bufsize: src.len() };
        self.raw.payload = payl.as_mut_ptr();
    }
}

impl<'a, H: HashSelect> BlankTask<'a, MemcopyHash<H>> {
    /// Set source, destination and payload buffers.
    ///
    /// # Panic
    ///
    /// This function panics if the payload buffer is not large enough.
    pub fn set_buffers(&mut self, src: CopySource<'a>, dst: &'a mut [u8], payl: &'a mut [u8]) {
        assert!(payl.len() >= H::PAYLOAD_BYTES);
        if let CopySource::ConstantFill(_) = src {
            self.raw.control0.constant_fill();
        }

        self.raw.source = src.into();
        self.raw.dest = dst.as_mut_ptr();
        self.raw.payload = payl.as_mut_ptr();
    }
}

impl<'a, C: CipherSelect, H: HashSelect> BlankTask<'a, CipherHash<C, H>> {
    /// Set source, destination and payload buffers.
    ///
    /// # Panic
    ///
    /// This function panics if the payload buffer is not large enough.
    pub fn set_buffers(&mut self, mem: CryptMem<'a>, payl: &'a mut [u8]) {
        assert!(payl.len() >= C::PAYLOAD_BYTES + H::PAYLOAD_BYTES);
        let len = match &mem {
            CryptMem::SourceDest(s, d) => {
                // There might be a better way than panicking, like returning a result
                assert_eq!(s.len(), d.len());
                s.len()
            }
            CryptMem::InPlace(sd) => sd.len(),
        };
        let (source, dest) = mem.into();

        self.raw.source = source;
        self.raw.dest = dest;
        self.raw.bufsize = BufSize { bufsize: len };
        self.raw.payload = payl.as_mut_ptr();
    }
}

enum TaskState {
    Idle,
    Running,
    Done,
}

/// An executable DCP work packet.
///
/// Polling it without calling start first will always return [WouldBlock](nb::Error::WouldBlock)
pub struct Task<'a, E> {
    ex: &'a mut E,
    state: TaskState,
    inner: ControlPacket<'a>,
    _unpin: PhantomPinned,
}

impl<E: Executor> Task<'_, E> {
    /// Checks if work on the packet has terminated
    pub fn poll(self: Pin<&mut Self>) -> crate::Result {
        let this = unsafe { self.get_unchecked_mut() };
        match this.state {
            TaskState::Idle => {
                if let Err(e) = this.ex.exec_one(&mut this.inner) {
                    return Err(nb::Error::Other(crate::Error::Executor(e)))
                }
                this.state = TaskState::Running;
                Err(nb::Error::WouldBlock)
            },
            TaskState::Running => {
                match this.inner.status.poll() {
                    crate::Result::Ok(r) => {
                        this.state = TaskState::Done;
                        return Ok(r)
                    }
                    crate::Result::Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
                    crate::Result::Err(e) => {
                        this.state = TaskState::Done;
                        return Err(e)
                    }
                }
            }
            TaskState::Done => return this.inner.status.poll()
        }
    }
}

impl<'a, E> Deref for Task<'a, E> {
    type Target = ControlPacket<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, E> DerefMut for Task<'a, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
