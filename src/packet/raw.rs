//! The packet passed to the hardware and its raw bits.
//!
//! This module contains the [ControlPacket] struct that is passed to the hardware at the start of
//! the operation and the data it's made up of.  
//! All this is part of the public API to allow finer control over the packets and the usage of
//! DCP features not implemented in this library by skipping the provided abstractionsand
//! directly manipulating the packet.
//!
//! Usage of this module is not recommended as minor mistakes might lead to incorrect or undefined
//! behavior.

use bitvec::prelude::*;
use crate::Error;

/// The struct that is passed to the DCP.
#[repr(C)]
pub struct ControlPacket<'a> {
    pub next: Option<&'a mut ControlPacket<'a>>,
    pub control0: Control0,
    pub control1: Control1,
    pub source: Source,
    pub dest: *mut (),
    pub bufsize: BufSize,
    pub payload: *mut (),
    pub status: Status,
}

/// The Control0 field of the control packet.   
/// It controls the main functions of the DCP and has a tag to identify packets.
#[repr(packed)]
#[derive(Default)]
pub struct Control0 {
    pub flags: BitArray<Lsb0, [u8; 3]>,
    pub tag: u8,
}

macro_rules! ctl0_flags {
    // Contructor(s) (f) with preset bits $x in range $r
	( new: $r:expr; $( $f:ident $x:expr ),+ ) => {
        $(
        pub fn $f() -> Self {
            let mut me = Self::default();
            me.flags[$r].store($x);
            me
        }
        )+
	};
    // Declare function(s) $f a single bit $p to 1
    ( $( $f:ident $p:expr ),+ ) => {
        $(
        pub fn $f(&mut self) {
            self.flags[$p..$p+1].store(1u8);
        }
        )+
    }
}

/// Has contructors for common operations and methods to set each flag to true.
impl Control0 {
    // idk if making these constructors is really worth it since other bits are going to be
    // set manually.
    // on the other hand there are only so many valid operations
    ctl0_flags!(new: 4..8;
        memcopy 0b0001u8,
        blit    0b1000u8,
        cipher  0b0010u8,
        hash    0b0100u8,
        memcopy_hash 0b101u8,
        cipher_hash 0b110u8
    );

    ctl0_flags!(
        interrupt_enable 0,
        decr_semaphore 1,
        chain 2,
        chain_continuous 3,
        enable_memcopy 4,
        enable_cipher 5,
        enable_hash 6,
        enable_blit 7,
        cipher_encrypt 8,
        cipher_init 9,
        otp_key 10,
        payload_key 11,
        hash_init 12,
        hash_term 13,
        hash_check 14,
        hash_output 15,
        constant_fill 16,
        test_sema_irq 17,
        key_byteswap 18,
        key_wordswap 19,
        input_byteswap 20,
        input_wordswap 21,
        output_byteswap 22,
        output_wordswap 23
    );
}

/// The Control1 field contains values used in encrypt, hash or blit operations.
#[derive(Default)]
#[repr(transparent)]
pub struct Control1(BitArray<Lsb0, u32>);

macro_rules! ctl1_flags {
    // Create function $f that stores up to a byte in range $r
	( $( $f:ident $r:expr ),+ ) => {
		$(
        pub fn $f(&mut self, bits: u8) {
            self.0[$r].store(bits)
        }
        )+
	};
}

/// The provided methods can be used to set the values of sections of the packet.
impl Control1 {
    /// Creates Control1 packet for a blit operation taking the framebuffer lenght in (width) in
    /// bytes.
    #[allow(dead_code)]
    pub fn blit(fblen: u16) -> Self {
        let mut me = Self::default();
        me.0[0..17].store(fblen);
        me
    }

    ctl1_flags!(
        cipher_select 0..4,
        cipher_mode 4..8,
        key_select 8..16,
        hash_select 16..20,
        cipher_config 24..32
    );
}

/// The source buffer.
/// It can be a 32 bit value for constant fill or a pointer.
#[repr(C)]
#[derive(Clone, Copy)]
pub union Source {
    pub constant: u32,
    pub pointer: *const ()
}

impl From<&[u8]> for Source {
    fn from(slice: &[u8]) -> Self {
        Self { pointer: slice as *const [u8] as *const () }
    }
}

/// Holds the buffer size or the blit framebuffer's height and width.
#[derive(Clone, Copy)]
#[repr(C)]
pub union BufSize {
    pub bufsize: usize,
    pub blit: BlitSize
}

/// Holds the blit framebuffer size data.
#[derive(Clone, Copy)]
#[repr(packed)]
pub struct BlitSize {
    /// Width in bytes
    pub width: u16,
    /// Height in lines
    pub height: u16
}

/// Is filled by the DCP at the end of the operation, holds eventual errors and the packet tag.
#[derive(Default, Clone)]
#[repr(packed)]
pub struct Status {
    /// Completion or eventual errors.
    pub bits: BitArray<Lsb0, u8>,
    _pad: u8,
    pub error_code: u8,
    /// Tag initially put in Control0 for identification.
    pub tag: u8
}

impl Status {
    /// Non-blocking API to poll for completion.  
    /// Returns WouldBlock when the operation is not complete
    pub fn poll(&self) -> crate::Result {
        if self.bits[0] {
            match self.bits.as_buffer() {
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
