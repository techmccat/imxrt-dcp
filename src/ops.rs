//! DCP operation types
//!
//! This module contains the operations available to the DCP and traits to make writing this
//! library less of a pain. (TODO: actual documentation)

/// Memory copy operation.
///
/// Can be used to copy buffers or move memory pages around.
pub struct Memcopy;
/// Blit operation.
///
/// Copies R runs of C bytes to the target buffer.
pub struct Blit;
/// Symmetric block cipher operation.
///
/// Used to encrypt or decrypt data.
pub use crate::packet::Cipher;
/// One-way digest calculation.
pub use crate::packet::Hash;

/// Memcopy and hash in the same operation.
pub type MemcopyHash = (Memcopy, Hash);
/// Cipher and hash in the same operation.
/// 
/// The data can be hashed before or after the crypto operation.
pub type CipherHash = (Cipher, Hash);

/// Used to configure data swapping in the FIFOs.
pub enum SwapConfig {
    /// Assume data to be little-endian.
    Keep,
    /// Swap 4 byte words.
    WordSwap,
    /// Swap bytes.
    ByteSwap,
    /// Assume data to be big-endian.
    WordByteSwap,
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::Cipher {}
    impl Sealed for super::Hash {}
    impl Sealed for super::Memcopy {}
    impl Sealed for super::Blit {}
    impl<T: Sealed, U: Sealed> Sealed for (T, U) {}
}

/// Sealed trait implemented for hashing operations.
pub trait HasHash: private::Sealed {}
impl HasHash for Hash {}
impl HasHash for MemcopyHash {}
impl HasHash for CipherHash {}

/// Sealed trait implemented for cryptographic operations.
pub trait HasCrypt: private::Sealed {}
impl HasCrypt for Cipher {}
impl HasCrypt for CipherHash {}
