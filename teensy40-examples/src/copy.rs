#![no_std]
#![no_main]

use teensy4_bsp as bsp;
use teensy4_panic as _;

use cortex_m::{asm, delay::Delay, peripheral::syst::SystClkSource};
use imxrt_dcp::{
    ex::SingleChannel,
    ops::Memcopy,
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
    let mut dest_buf = [0u8; 64];

    let builder: PacketBuilder<Memcopy> = PacketBuilder::default()
        .tag(7)
        .source(Source {
            pointer: &src_buf[0] as *const u8,
        })
        .dest(&mut dest_buf)
        .decr_semaphore();

    let mut packet: ControlPacket = builder.into();
    log::info!("Queueing work packet on the DCP");
    let task = ex.exec_one(&mut packet).unwrap();

    let res = imxrt_dcp::block!(task.poll());
    log::warn!("Operation result: {res:?}");

    if src_buf == dest_buf {
        log::info!("Buffers match, some mysterious entity has copied 64 bytes.")
    } else {
        log::error!("Buffers don't match. What the fuck.")
    }

    loop {
        asm::nop()
    }
}
