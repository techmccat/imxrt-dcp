pub mod raw;

use core::marker::PhantomData;

use crate::task::BlankTask;

use super::ops::{*, config::*};
use raw::*;

/// Constructs a control packet for the given operation.
///
/// The options will be different based on the operation.
pub struct PacketBuilder<'a, T> {
    pub raw: ControlPacket<'a>, phantom: PhantomData<T>,
}

impl PacketBuilder<'_, Memcopy> {
    pub fn new() -> Self {
        let raw = ControlPacket {
            control0: Control0::memcopy(),
            control1: Control1::default(),
            ..Default::default()
        };

        Self {
            raw,
            phantom: PhantomData,
        }
    }
}

impl PacketBuilder<'_, Blit> {
    pub fn new() -> Self {
        let raw = ControlPacket {
            control0: Control0::blit(),
            control1: Control1::default(),
            ..Default::default()
        };

        Self {
            raw,
            phantom: PhantomData,
        }
    }
}

impl<C: CipherSelect> PacketBuilder<'_, Cipher<C>> {
    pub fn new() -> Self {
        let raw = ControlPacket {
            control0: Control0::cipher(),
            control1: C::ctl1(),
            ..Default::default()
        };

        Self {
            raw,
            phantom: PhantomData,
        }
    }
}

impl<H: HashSelect> PacketBuilder<'_, Hash<H>> {
    pub fn new() -> Self {
        let raw = ControlPacket {
            control0: Control0::hash(),
            control1: H::ctl1(),
            ..Default::default()
        };

        Self {
            raw,
            phantom: PhantomData,
        }
    }
}

impl<H: HashSelect> PacketBuilder<'_, MemcopyHash<H>> {
    pub fn new() -> Self {
        let raw = ControlPacket {
            control0: Control0::memcopy_hash(),
            control1: H::ctl1(),
            ..Default::default()
        };

        Self {
            raw,
            phantom: PhantomData,
        }
    }
}

impl<C: CipherSelect, H: HashSelect> PacketBuilder<'_, CipherHash<C, H>> {
    pub fn new() -> Self {
        let mut control1 = Control1::default();
        control1.cipher_select(C::CIPHER_SELECT);
        control1.cipher_mode(C::CIPHER_MODE);
        control1.hash_select(H::HASH_SELECT);

        let raw = ControlPacket {
            next: None,
            control0: Control0::cipher_hash(),
            control1,
            ..Default::default()
        };

        Self {
            raw,
            phantom: PhantomData,
        }
    }
}

macro_rules! ctl0_wrapper {
	( $fn:ident ) => {
        pub fn $fn(&mut self) {
            self.raw.control0.$fn()
        }
	};

	( $fn:ident => $( $raw:ident ),+ ) => {
        pub fn $fn(&mut self) {
            $(
            self.raw.control0.$raw();
            )+
        }
	};
}

impl<T: HasCrypt> PacketBuilder<'_, T> {
    pub fn set_key(&mut self, key: CryptKey) {
        match key {
            CryptKey::Payload => self.raw.control0.payload_key(),
            CryptKey::KeyRam(_) => (),
            _ => self.raw.control0.otp_key()
        }

        self.raw.control1.key_select(key.into())
    }

    ctl0_wrapper!(encrypt => cipher_encrypt);
    ctl0_wrapper!(cipher_init);
}

impl<T: HasHash> PacketBuilder<'_, T> {
    ctl0_wrapper!(hash_init);
    ctl0_wrapper!(hash_term);
    ctl0_wrapper!(check_hash => hash_check);
    ctl0_wrapper!(hash_output);
}

impl<'a, T> PacketBuilder <'a, T> {
    ctl0_wrapper!(input_big_endian => input_wordswap, input_byteswap);
    ctl0_wrapper!(output_big_endian => output_wordswap, output_byteswap);
    ctl0_wrapper!(key_big_endian => key_wordswap, key_byteswap);
    ctl0_wrapper!(chain_continuous);
    ctl0_wrapper!(decr_semaphore);

    pub fn new_task(&self) -> BlankTask<'a, T> {
        BlankTask {
            raw: self.raw.clone(),
            _op: self.phantom
        }
    }
}
