use super::{BlitSize, BufSize, Cipher, Control0Flag, ControlPacket, Hash, KeySelect, Source};
use crate::ops::*;
use core::{marker::PhantomData, mem::zeroed};

/// Constructs a control packet for the given operation.
///
/// The options will be different based on the operation.
pub struct PacketBuilder<'a, T> {
    raw: ControlPacket<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T> PacketBuilder<'a, T> {
    /// Set the source buffer or constant for the operation
    pub fn source(mut self, source: Source<'a>) -> Self {
        self.raw.source = source;
        self
    }

    /// Set the destination buffer for the operation
    ///
    /// # Safety
    ///
    /// The destination buffer lenght must be lower than or
    /// equal to the source buffer size to prevent out of
    /// bounds access.
    pub fn dest(mut self, slice: &'a mut [u8]) -> Self {
        self.raw.dest = slice as *mut [u8] as *mut u8;
        self.raw.bufsize = BufSize {
            buf: slice.len() as u32,
        };
        self
    }

    /// Set the payload buffer for the operation
    ///
    /// # Safety
    ///
    /// The payload must hold whatever the operation requires,
    /// for a maximum of 64 bytes.
    /// AES CBC cipher init takes a 16 byte IV,
    /// HashTerm needs 20B for SHA1 or 32 for SHA2 and the expected
    /// hash is read from there if the HashCheck flag is set.
    pub fn payload(mut self, slice: &'a mut [u8]) -> Self {
        self.raw.payload = slice as *mut [u8] as *mut u8;
        self
    }

    /// Set the packet tag.
    pub fn tag(mut self, tag: u8) -> Self {
        self.raw.control0.tag = tag;
        self
    }

    /// Configure byte swapping in the input.
    pub fn input_swap(mut self, conf: SwapConfig) -> Self {
        let ctl0 = self.raw.control0;
        self.raw.control0 = match conf {
            SwapConfig::Keep => ctl0,
            SwapConfig::WordSwap => ctl0.flag(Control0Flag::InputWordSwap),
            SwapConfig::ByteSwap => ctl0.flag(Control0Flag::InputByteSwap),
            SwapConfig::WordByteSwap => ctl0
                .flag(Control0Flag::InputWordSwap)
                .flag(Control0Flag::InputByteSwap),
        };
        self
    }

    /// Configure byte swapping in the output.
    pub fn output_swap(mut self, conf: SwapConfig) -> Self {
        let ctl0 = self.raw.control0;
        self.raw.control0 = match conf {
            SwapConfig::Keep => ctl0,
            SwapConfig::WordSwap => ctl0.flag(Control0Flag::OutputWordSwap),
            SwapConfig::ByteSwap => ctl0.flag(Control0Flag::OutputByteSwap),
            SwapConfig::WordByteSwap => ctl0
                .flag(Control0Flag::OutputWordSwap)
                .flag(Control0Flag::OutputByteSwap),
        };
        self
    }

    /// Decrement DCP channel semaphore when done.
    /// Enable on the last packet of a chain.
    pub fn decr_semaphore(mut self) -> Self {
        self.raw.control0 = self.raw.control0.flag(Control0Flag::DecrSemaphore);
        self
    }

    /// Fire a DCP_IRQ interrupt on operation completion.
    pub fn interrupt_enable(mut self) -> Self {
        self.raw.control0 = self.raw.control0.flag(Control0Flag::InterruptEnable);
        self
    }
}

impl<'a, T> From<PacketBuilder<'a, T>> for ControlPacket<'a> {
    fn from(builder: PacketBuilder<'a, T>) -> Self {
        builder.raw
    }
}

impl<'a> PacketBuilder<'a, Cipher> {
    pub fn new() -> Self {
        let mut raw: ControlPacket = unsafe { zeroed() };
        raw.control0 = raw
            .control0
            .flag(Control0Flag::EnableCipher)
            .flag(Control0Flag::PayloadKey);
        Self {
            raw,
            _marker: PhantomData,
        }
    }
}

impl<'a> Default for PacketBuilder<'a, Cipher> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PacketBuilder<'a, Hash> {
    pub fn new() -> Self {
        let mut raw: ControlPacket = unsafe { zeroed() };
        raw.control0 = raw.control0.flag(Control0Flag::EnableHash);
        Self {
            raw,
            _marker: PhantomData,
        }
    }
}

impl<'a> Default for PacketBuilder<'a, Hash> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PacketBuilder<'a, Memcopy> {
    pub fn new() -> Self {
        let mut raw: ControlPacket = unsafe { zeroed() };
        raw.control0 = raw.control0.flag(Control0Flag::EnableMemcopy);
        Self {
            raw,
            _marker: PhantomData,
        }
    }
}

impl<'a> Default for PacketBuilder<'a, Memcopy> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PacketBuilder<'a, Blit> {
    pub fn new() -> Self {
        let mut raw: ControlPacket = unsafe { zeroed() };
        raw.control0 = raw.control0.flag(Control0Flag::EnableBlit);
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    /// Set the destination framebuffer.
    ///
    /// Takes an output buffer and a line width in bytes as input.
    pub fn framebuffer(mut self, buf: &'a mut [u8], width: u16) -> Self {
        self.raw.dest = buf as *mut [u8] as *mut u8;
        self.raw.bufsize = BufSize {
            blit: BlitSize {
                width,
                height: (buf.len() / width as usize) as u16,
            },
        };
        self.raw.control1.blit_size = buf.len() as u16;
        self
    }
}

impl<'a> Default for PacketBuilder<'a, Blit> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PacketBuilder<'a, MemcopyHash> {
    pub fn new() -> Self {
        let mut raw: ControlPacket = unsafe { zeroed() };
        raw.control0 = raw
            .control0
            .flag(Control0Flag::EnableHash)
            .flag(Control0Flag::EnableMemcopy);
        Self {
            raw,
            _marker: PhantomData,
        }
    }
}

impl<'a> Default for PacketBuilder<'a, MemcopyHash> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PacketBuilder<'a, CipherHash> {
    pub fn new() -> Self {
        let mut raw: ControlPacket = unsafe { zeroed() };
        raw.control0 = raw
            .control0
            .flag(Control0Flag::EnableCipher)
            .flag(Control0Flag::EnableMemcopy);
        Self {
            raw,
            _marker: PhantomData,
        }
    }
}

impl<'a> Default for PacketBuilder<'a, CipherHash> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: HasHash> PacketBuilder<'a, T> {
    /// Set the hashing algorhitm.
    pub fn hash(mut self, hash: Hash) -> Self {
        self.raw.control1.crypto.hash = hash;
        self
    }

    /// Initialize the hashing operation.
    /// Needed when hasshing the first block of a series.
    pub fn hash_init(mut self) -> Self {
        self.raw.control0 = self.raw.control0.flag(Control0Flag::HashInit);
        self
    }

    /// Terminate the hashing operation and write the hash to the payload.
    pub fn hash_term(mut self) -> Self {
        self.raw.control0 = self.raw.control0.flag(Control0Flag::HashTerm);
        self
    }

    /// Check that the calculated hash matches the one provided in the payload.
    pub fn hash_check(mut self) -> Self {
        self.raw.control0 = self.raw.control0.flag(Control0Flag::HashCheck);
        self
    }
}

impl<'a, T: HasCrypt> PacketBuilder<'a, T> {
    /// Perform encryption in-place, without separate source and destination buffers
    pub fn in_place(self, buf: &mut [u8]) -> Self {
        let ptr = buf as *mut [u8] as *mut u8;
        Self {
            raw: ControlPacket {
                source: Source {
                    pointer: ptr as *const u8,
                },
                dest: ptr,
                bufsize: BufSize {
                    buf: buf.len() as u32,
                },
                ..self.raw
            },
            ..self
        }
    }

    /// Select the encryption algorhitm.
    pub fn cipher(mut self, cipher: Cipher) -> Self {
        self.raw.control1.crypto.cipher = cipher;
        self
    }

    /// Select the source for the encryption key.
    pub fn key(mut self, key: KeySelect) -> Self {
        self.raw.control1.crypto.key = key;
        self
    }

    /// Initialize the cipher (get IV from payload if using AES CBC).
    pub fn cipher_init(mut self) -> Self {
        self.raw.control0 = self.raw.control0.flag(Control0Flag::CipherInit);
        self
    }

    /// Encrypt the data (defaults to decryption).
    pub fn encrypt(mut self) -> Self {
        self.raw.control0 = self.raw.control0.flag(Control0Flag::CipherEncrypt);
        self
    }

    /// Configure data swapping in the key in the payload section.
    pub fn key_swap(mut self, conf: SwapConfig) -> Self {
        let ctl0 = self.raw.control0;
        self.raw.control0 = match conf {
            SwapConfig::Keep => ctl0,
            SwapConfig::WordSwap => ctl0.flag(Control0Flag::OutputWordSwap),
            SwapConfig::ByteSwap => ctl0.flag(Control0Flag::OutputByteSwap),
            SwapConfig::WordByteSwap => ctl0
                .flag(Control0Flag::OutputWordSwap)
                .flag(Control0Flag::OutputByteSwap),
        };
        self
    }
}
