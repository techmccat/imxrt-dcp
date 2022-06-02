use crate::Error;
use core::marker::PhantomData;

pub mod builder;

/// The struct that is passed to the DCP.
#[derive(Debug)]
#[repr(C)]
pub struct ControlPacket<'a> {
    next: *mut ControlPacket<'a>,
    pub(crate) control0: Control0,
    control1: Control1,
    source: Source<'a>,
    dest: *mut u8,
    bufsize: BufSize,
    payload: *mut u8,
    pub(crate) status: Status,
    _lifetime: PhantomData<&'a ()>,
}

/// The Control0 field of the control packet.   
/// It controls the main functions of the DCP and has a tag to identify packets.
#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub(crate) struct Control0 {
    flags: [u8; 3],
    tag: u8,
}

/// Flags that can be set in the Control0 field
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub(crate) enum Control0Flag {
    InterruptEnable = 1,
    DecrSemaphore = 1 << 1,
    Chain = 1 << 2,
    ChainContinuous = 1 << 3,
    EnableMemcopy = 1 << 4,
    EnableCipher = 1 << 5,
    EnableHash = 1 << 6,
    EnableBlit = 1 << 7,
    CipherEncrypt = 1 << 8,
    CipherInit = 1 << 9,
    OtpKey = 1 << 10,
    PayloadKey = 1 << 11,
    HashInit = 1 << 12,
    HashTerm = 1 << 13,
    HashCheck = 1 << 14,
    HashOutput = 1 << 15,
    ConstantFill = 1 << 16,
    TestSemaIRQ = 1 << 17,
    KeyByteSwap = 1 << 18,
    KeyWordSwap = 1 << 19,
    InputByteSwap = 1 << 20,
    InputWordSwap = 1 << 21,
    OutputByteSwap = 1 << 22,
    OutputWordSwap = 1 << 23,
}

impl Control0 {
    pub(crate) fn flag(mut self, flag: Control0Flag) -> Self {
        let ptr = &mut self as *mut Self as *mut u32;
        unsafe { *ptr |= flag as u32 };
        self
    }
}

/// The Control1 field contains values used in encrypt, hash or blit operations.
#[derive(Clone, Copy)]
#[repr(C)]
union Control1 {
    /// Crypto config
    pub crypto: Ctl1Crypto,
    /// Total framebuffer lenght
    pub blit_size: u16,
}

impl core::fmt::Debug for Control1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let blit = unsafe { self.blit_size };
        let crypto = unsafe { self.crypto };
        f.write_fmt(format_args!(
            "Control1 {blit} bytes or {crypto:#?}"
        ))
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Ctl1Crypto {
    cipher: Cipher,
    key: KeySelect,
    hash: Hash,
    // doesn't seem to be used anywhere
    _cipher_config: u8,
}

/// Supported symmetric ciphers
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Cipher {
    Aes128Ecb = 0,
    Aes128Cbc = 1 << 3,
}

/// Select key to use from a keyslot
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum KeySelect {
    Key0 = 0x0,
    Key1 = 0x1,
    Key2 = 0x2,
    Key3 = 0x3,
    UniqueKey = 0xFE,
    OtpKey = 0xFF,
}

/// Supported hashing algorithms
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Hash {
    Sha1 = 0,
    Crc32 = 1,
    Sha256 = 2,
}

/// Data source for the DCP.
///
/// It can either be a 32 bit value for constant fill or a pointer.
#[repr(C)]
#[derive(Clone, Copy)]
pub union Source<'a> {
    pub constant: u32,
    pub pointer: *const u8,
    _lifetime: PhantomData<&'a ()>
}

impl core::fmt::Debug for Source<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "Source {:#x}", unsafe { self.constant }
        ))
    }
}

/// Holds the buffer size or the blit framebuffer's height and width.
#[derive(Clone, Copy)]
#[repr(C)]
union BufSize {
    pub buf: u32,
    pub blit: BlitSize,
}

impl core::fmt::Debug for BufSize {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "BufSize ({})", unsafe { self.buf }
        ))
    }
}

/// Holds the blit framebuffer size data.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct BlitSize {
    /// Width in bytes
    pub width: u16,
    /// Height in lines
    pub height: u16,
}

/// Is filled by the DCP at the end of the operation, holds eventual errors and the packet tag.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Status {
    /// Completion or eventual errors.
    pub bits: u8,
    _pad: u8,
    pub error_code: u8,
    /// Tag initially put in Control0 for identification.
    pub tag: u8,
}

impl Status {
    /// Non-blocking API to poll for completion.  
    /// Returns WouldBlock when the operation is not complete
    pub fn poll(&self) -> crate::Result {
        if self.bits & 1 == 1 {
            match self.bits {
                1 => Ok(self.tag),
                2 => Err(nb::Error::Other(Error::HashMismatch(self.error_code))),
                4 => Err(nb::Error::Other(Error::SetupError(self.error_code))),
                8 => Err(nb::Error::Other(Error::PacketError(self.error_code))),
                16 => Err(nb::Error::Other(Error::SourceError(self.error_code))),
                32 => Err(nb::Error::Other(Error::DestError(self.error_code))),
                _ => Err(nb::Error::Other(Error::Other(self.error_code))),
            }
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}
