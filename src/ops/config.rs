use crate::packet::raw::*;

/// Source for a copy or blit operation.
///
/// It can be a pointer to a memory buffer or a 32 bit word for constant fill
pub enum CopySource<'a> {
    MemoryBuffer(&'a [u8]),
    ConstantFill(u32),
}

impl<'a> Into<Source<'a>> for CopySource<'a> {
    fn into(self) -> Source<'a> {
        match self {
            Self::MemoryBuffer(slice) => Source::from(slice),
            Self::ConstantFill(constant) => Source { constant },
        }
    }
}

/// Destination buffer for a blit operation.
///
/// It holds a mutable reference to the buffer and the line width
pub struct Framebuffer<'a> {
    pub pointer: &'a mut [u8],
    pub width: u16,
}

impl Framebuffer<'_> {
    pub fn height(&self) -> u16 {
        self.pointer.len() as u16 / self.width
    }
}

/// Source and target buffers for crypto operations.
///
/// Crypt operations can be run in-place, without the need to allocate a separate source and
/// destination buffer.
pub enum CryptMem<'a> {
    InPlace(&'a mut [u8]),
    SourceDest(&'a [u8], &'a mut [u8])
}

impl<'s> Into<(Source<'s>, *mut u8)> for CryptMem<'s> {
    fn into(self) -> (Source<'s>, *mut u8) {
        match self {
            Self::SourceDest(s, d) => (Source::from(s), d.as_mut_ptr()),
            Self::InPlace(sd) => {
                let d = sd.as_mut_ptr();
                let s = Source { pointer: d as *const () };
                (s, d)
            }
        }
    }
}

pub enum CryptKey {
    Payload,
    KeyRam(u8),
    Unique,
    Otp(bool),
}

impl Into<u8> for CryptKey {
    fn into(self) -> u8 {
        match self {
            Self::Payload => 0,
            Self::KeyRam(n) => n,
            Self::Unique => 0xfe,
            Self::Otp(_) => 0xff,
        }
    }
}
