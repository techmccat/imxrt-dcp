//! This module contains the operations available to the DCP and traits to make writing this
//! library less of a pain. (TODO: actual documentation)

use core::marker::PhantomData;
use crate::packet::raw::*;

pub mod config;

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
pub struct Cipher<C: CipherSelect> {
    phantom: PhantomData<C>
}
/// One-way digest calculation.
pub struct Hash<S: HashSelect> {
    phantom: PhantomData<S>
}
/// Memcopy and hash in the same operation.
pub struct MemcopyHash<S: HashSelect> {
    phantom: PhantomData<S>
}
/// Cipher and hash in the same operation.
/// 
/// The data can be hashed before or after the crypto operation.
pub struct CipherHash<C: CipherSelect, H: HashSelect> {
    cipher: PhantomData<C>,
    hash: PhantomData<H>
}

pub struct Sha1;
pub struct Sha256;
pub struct Crc32;

/// Common trait for available hashes.
pub unsafe trait HashSelect {
    const HASH_SELECT: u8;
    const PAYLOAD_BYTES: usize;

    fn ctl1() -> Control1 {
        let mut new = Control1::default();
        new.hash_select(Self::HASH_SELECT);
        new
    }
}

unsafe impl HashSelect for Sha1 {
    const HASH_SELECT: u8 = 0;
    const PAYLOAD_BYTES: usize = 20;
}

unsafe impl HashSelect for Sha256 {
    const HASH_SELECT: u8 = 2;
    const PAYLOAD_BYTES: usize = 20;
}

unsafe impl HashSelect for Crc32 {
    const HASH_SELECT: u8 = 1;
    const PAYLOAD_BYTES: usize = 4;
}

pub struct Aes128Ecb;
pub struct Aes128Cbc;

/// Common trait for available ciphers.
pub unsafe trait CipherSelect {
    const CIPHER_SELECT: u8;
    const CIPHER_MODE: u8;
    const PAYLOAD_BYTES: usize;

    fn ctl1() -> Control1 {
        let mut new = Control1::default();
        new.cipher_select(Self::CIPHER_SELECT);
        new.cipher_mode(Self::CIPHER_MODE);
        new
    }
}

unsafe impl CipherSelect for Aes128Ecb {
    const CIPHER_SELECT: u8 = 0;
    const CIPHER_MODE: u8 = 0;
    const PAYLOAD_BYTES: usize = 16;
}

unsafe impl CipherSelect for Aes128Cbc {
    const CIPHER_SELECT: u8 = 0;
    const CIPHER_MODE: u8 = 1;
    const PAYLOAD_BYTES: usize = 32;
}

/// Marker trait for common hash options.
pub trait HasHash {}
/// Marker trait for common cipher options.
pub trait HasCrypt {}

impl<H: HashSelect> HasHash for Hash<H> {}
impl<H: HashSelect> HasHash for MemcopyHash<H> {}
impl<H: HashSelect, C: CipherSelect> HasHash for CipherHash<C, H> {}

impl<C: CipherSelect> HasCrypt for Cipher<C> {}
impl<H: HashSelect, C: CipherSelect> HasCrypt for CipherHash<C, H> {}
