#![no_std]
#![no_main]

use teensy4_bsp as bsp;
use teensy4_panic as _;

use cortex_m::{asm, delay::Delay, peripheral::syst::SystClkSource};
use imxrt_dcp::{
    ex::SingleChannel,
    ops::Hash,
    packet::{ControlPacket, Source},
    prelude::*,
};
use teensy40_examples::logging;

#[cortex_m_rt::entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let ip = bsp::Peripherals::take().unwrap();
    let mut delay = Delay::with_source(cp.SYST, bsp::EXT_SYSTICK_HZ, SystClkSource::External);
    let mut ccm = ip.ccm.handle;

    logging::init().unwrap();
    delay.delay_ms(2000);

    // let dcp = _setup_dcp(&mut ccm);
    let dcp = dcp::Unclocked::take().unwrap().clock(ccm.raw().0).build();
    let ex: SingleChannel<Ch0> = SingleChannel::take(dcp).unwrap();
    log::info!("DCP Init done");

    let mut src_buf = [0u8; 64];
    for i in 0..64 {
        src_buf[i] = i as u8;
    }
    // stores calculated CRC32 hash
    let mut dest_buf = [0u8; 4];
    // calculated with http://www.sunshine2k.de/coding/javascript/crc/crc_js.html
    // Options:
    // Input reflected:     false
    // Result reflected:    false
    // Polynomial:          0x4C11DB7
    // Initial value:       0xFFFFFFFF
    // Final XOR value:     0x000000000
    // (Little endian)
    let expected_crc = 0xBCBD08F5u32;

    {
        let builder: PacketBuilder<Hash> = PacketBuilder::default()
            .hash(Hash::Crc32)
            .hash_init()
            .hash_term()
            .tag(7)
            .source(Source {
                pointer: &src_buf[0] as *const u8,
            })
            .payload(&mut dest_buf)
            .decr_semaphore();

        let mut packet: ControlPacket = builder.into();
        log::info!("Queueing work packet on the DCP");
        let task = ex.exec_one(&mut packet).unwrap();

        let res = imxrt_dcp::block!(task.poll());
        log::warn!("Operation result: {res:?}");
    }

    log::info!("Calculatec CRC = {dest_buf:X?}");
    log::info!("Expected CRC   = {:X?}", expected_crc.to_le_bytes());
    if dest_buf == expected_crc.to_le_bytes() {
        log::info!("Buffers match, CRC worked as expected.")
    } else {
        log::error!("Buffers don't match.");
    }

    loop {
        asm::wfi()
    }
}
