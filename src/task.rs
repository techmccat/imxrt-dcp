use core::{marker::PhantomData, pin::Pin};

use crate::{ex::Executor, packet::raw::{ControlPacket, BufSize, BlitSize, Control1}, ops::{*, config::*}};

// TODO: set buffers when constructing
/// A [`Task`] whose buffers and tag have not been set yet.
///
/// Once the buffers (and optionally the tag) have been set it can be frozen into an executable
/// [`Task`].
pub struct BlankTask<'a, O> {
    pub(crate) raw: ControlPacket<'a>,
    pub(crate) _op: PhantomData<O>
}

impl<'a, O> BlankTask<'a, O> {
    pub fn set_tag(&mut self, tag: u8) {
        self.raw.status.tag = tag;
    }

    pub fn freeze(self) -> Task<'a> {
        Task {
            inner: self.raw
        }
    }
}

impl<'a> BlankTask<'a, Memcopy> {
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
    pub fn set_buffers(&mut self, src: CopySource<'a>, dst: Framebuffer<'a>) {
        if let CopySource::ConstantFill(_) = src {
            self.raw.control0.constant_fill();
        }

        self.raw.control1 = Control1::blit(dst.width);        
        self.raw.source = src.into();
        self.raw.dest = dst.pointer.as_mut_ptr();
        self.raw.bufsize = BufSize { blit: BlitSize { width: dst.width, height: dst.height() } };
    }
}

// TODO: Alignment optimizations, might want to look into https://crates.io/crates/aligned
impl<'a, C: CipherSelect> BlankTask<'a, Cipher<C>> {
    pub fn set_buffers(&mut self, mem: CryptMem<'a>, payl: &'a mut [u8]) {
        assert!(payl.len() >= C::PAYLOAD_BYTES);

        let len = match &mem {
            CryptMem::SourceDest(s, d) => {
            // There might be a better way than panicking, like returning a result
            assert_eq!(s.len(), d.len());
            s.len() }
            CryptMem::InPlace(sd) => sd.len()
        };

        let (source, dest) = mem.into();

        self.raw.source = source;
        self.raw.dest = dest;
        self.raw.bufsize = BufSize { bufsize: len };
        self.raw.payload = payl.as_mut_ptr()
    }
}

impl<'a, H: HashSelect> BlankTask<'a, Hash<H>> {
    pub fn set_buffers(&mut self, src: &'a [u8], payl: &'a mut [u8]) {
        assert!(payl.len() >= H::PAYLOAD_BYTES);

        self.raw.source = src.into();
        self.raw.bufsize = BufSize { bufsize: src.len() };
        self.raw.payload = payl.as_mut_ptr();
    }
}

impl<'a, H: HashSelect> BlankTask<'a, MemcopyHash<H>> {
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
    pub fn set_buffers(&mut self, mem: CryptMem<'a>, payl: &'a mut [u8]) {
        assert!(payl.len() >= C::PAYLOAD_BYTES + H::PAYLOAD_BYTES);
        let len = match &mem {
            CryptMem::SourceDest(s, d) => {
            // There might be a better way than panicking, like returning a result
                assert_eq!(s.len(), d.len());
                s.len()
            }
            CryptMem::InPlace(sd) => sd.len()
        };
        let (source, dest) = mem.into();

        self.raw.source = source;
        self.raw.dest = dest;
        self.raw.bufsize = BufSize { bufsize: len };
        self.raw.payload = payl.as_mut_ptr();
    }
}

/// An executable DCP work packet.
///
/// Polling it without calling start first will always return [WouldBlock](nb::Error::WouldBlock)
pub struct Task<'a> {
    inner: ControlPacket<'a>,
}

impl Task<'_> {
    /// Tells the executor to start/schedule the task for execution
    pub fn start<E: Executor>(self: Pin<&mut Self>, ex: &mut E) {
        ex.exec(self)
    }

    /// Checks if work on the packet has terminated
    pub fn poll(self: Pin<&Self>) -> crate::Result {
        self.inner.status.poll()
    }
}
