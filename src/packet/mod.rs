pub mod raw;

use core::marker::PhantomData;
use core::ptr::null_mut;

use super::ops::{*, config::*};
use raw::*;

/// Constructs a control packet for the given operation.
///
/// The options will be different based on the operation.
pub struct PacketBuilder<'a, T> {
    pub raw: ControlPacket<'a>,
    phantom: PhantomData<T>,
}

impl PacketBuilder<'_, Memcopy> {
    pub fn new(src: CopySource, dst: &mut [u8]) -> Self {
        let mut control0 = Control0::memcopy();
        let control1 = Control1::default();

        if let CopySource::ConstantFill(_) = src {
            control0.constant_fill();
        }

        let raw = ControlPacket {
            next: None,
            control0,
            control1,
            source: src.into(),
            dest: dst as *mut [u8] as *mut (),
            bufsize: BufSize { bufsize: dst.len() },
            payload: null_mut(),
            status: Status::default(),
        };

        Self {
            raw,
            phantom: PhantomData,
        }
    }
}

impl PacketBuilder<'_, Blit> {
    pub fn new(src: CopySource, fb: Framebuffer) -> Self {
        let mut control0 = Control0::blit();
        let control1 = Control1::blit(fb.width);

        if let CopySource::ConstantFill(_) = src {
            control0.constant_fill();
        }

        let raw = ControlPacket {
            next: None,
            control0,
            control1,
            source: src.into(),
            dest: fb.pointer.as_mut() as *mut [u8] as *mut (),
            bufsize: BufSize { blit: BlitSize { width: fb.width, height: fb.height() } },
            payload: null_mut(),
            status: Status::default(),
        };

        Self {
            raw,
            phantom: PhantomData,
        }
    }
}

// TODO: Alignment optimizations, might want to look into https://crates.io/crates/aligned
impl<C: CipherSelect> PacketBuilder<'_, Cipher<C>> {
    pub fn new(mem: CryptMem, payl: &mut [u8]) -> Self {
        assert!(payl.len() >= C::PAYLOAD_BYTES);

        let len = match &mem {
            CryptMem::SourceDest(s, d) => {
            // There might be a better way than panicking, like returning a result
            assert_eq!(s.len(), d.len());
            s.len() }
            CryptMem::InPlace(sd) => sd.len()
        };

        let (source, dest) = mem.into();

        let raw = ControlPacket {
            next: None,
            control0: Control0::cipher(),
            control1: C::ctl1(),
            source,
            dest,
            bufsize: BufSize { bufsize: len },
            payload: payl as *mut [u8] as *mut (),
            status: Status::default(),
        };

        Self {
            raw,
            phantom: PhantomData,
        }
    }
}

impl<H: HashSelect> PacketBuilder<'_, Hash<H>> {
    pub fn new(src: &[u8], payl: &mut [u8]) -> Self {
        assert!(payl.len() >= H::PAYLOAD_BYTES);

        let raw = ControlPacket {
            next: None,
            control0: Control0::hash(),
            control1: H::ctl1(),
            source: src.into(),
            dest: null_mut(),
            payload: payl as *mut [u8] as *mut (),
            bufsize: BufSize { bufsize: src.len() },
            status: Status::default(),
        };

        Self {
            raw,
            phantom: PhantomData,
        }
    }
}

impl<H: HashSelect> PacketBuilder<'_, MemcopyHash<H>> {
    pub fn new(src: &[u8], payl: &mut [u8]) -> Self {
        // TODO: Decent way to create payload
        assert!(payl.len() >= H::PAYLOAD_BYTES);

        let raw = ControlPacket {
            next: None,
            control0: Control0::memcopy_hash(),
            control1: H::ctl1(),
            source: src.into(),
            dest: null_mut(),
            payload: payl as *mut [u8] as *mut (),
            bufsize: BufSize { bufsize: src.len() },
            status: Status::default(),
        };

        Self {
            raw,
            phantom: PhantomData,
        }
    }
}

impl<C: CipherSelect, H: HashSelect> PacketBuilder<'_, CipherHash<C, H>> {
    pub fn new(mem: CryptMem, payl: &mut [u8]) -> Self {
        // TODO: Decent way to create payload
        assert!(payl.len() >= C::PAYLOAD_BYTES + H::PAYLOAD_BYTES);

        let mut control1 = Control1::default();
        control1.cipher_select(C::CIPHER_SELECT);
        control1.cipher_mode(C::CIPHER_MODE);
        control1.hash_select(H::HASH_SELECT);

        let len = match &mem {
            CryptMem::SourceDest(s, d) => {
            // There might be a better way than panicking, like returning a result
            assert_eq!(s.len(), d.len());
            s.len() }
            CryptMem::InPlace(sd) => sd.len()
        };

        let (source, dest) = mem.into();

        let raw = ControlPacket {
            next: None,
            control0: Control0::cipher_hash(),
            control1,
            source,
            dest,
            payload: payl as *mut [u8] as *mut (),
            bufsize: BufSize { bufsize: len },
            status: Status::default(),
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

impl<T> PacketBuilder <'_, T> {
    ctl0_wrapper!(input_big_endian => input_wordswap, input_byteswap);
    ctl0_wrapper!(output_big_endian => output_wordswap, output_byteswap);
    ctl0_wrapper!(key_big_endian => key_wordswap, key_byteswap);
    ctl0_wrapper!(chain_continuous);
    ctl0_wrapper!(decr_semaphore);
}
