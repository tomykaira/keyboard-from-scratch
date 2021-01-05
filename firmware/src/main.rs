//#![deny(warnings)]
#![no_std]
#![no_main]

extern crate panic_halt;

use cortex_m_rt::entry;

#[allow(unused_imports)]
use cortex_m_semihosting::hprintln;

mod cursor;
mod descr;
mod gpio;
mod hid_keycodes;
mod key_stream;
mod matrix;
mod peer;
mod pma;
mod usb;
use key_stream::KeyStream;

use crate::usb::{CompositeConfigDescriptor, ControlState, USBKbd, CONFIG_DESCR, DEVICE_DESCR};
use stm32f1::stm32f103;

fn setup_clock(rcc: &stm32f103::RCC, flash: &stm32f103::FLASH) {
    rcc.cr.write(|w| w.hsion().set_bit());
    while rcc.cr.read().hsirdy().bit_is_clear() {}
    rcc.cfgr.write(|w| w.sw().hsi());

    rcc.cr.write(|w| w.hsion().set_bit().hseon().set_bit());
    while rcc.cr.read().hserdy().bit_is_clear() {}
    rcc.cfgr.write(|w| w.sw().hse());

    flash.acr.write(|w| unsafe { w.latency().bits(0b010) });

    rcc.cfgr.write(|w| {
        w.sw()
            .hse()
            .hpre()
            .div1()
            .adcpre()
            .div8()
            .ppre1()
            .div2()
            .ppre2()
            .div1()
            .pllmul()
            .mul9()
            .pllsrc()
            .hse_div_prediv()
            .pllxtpre()
            .div1()
    });

    rcc.cr
        .write(|w| w.hsion().set_bit().hseon().set_bit().pllon().set_bit());
    while rcc.cr.read().pllrdy().bit_is_clear() {}
    rcc.cfgr.write(|w| w.sw().pll());
}

#[entry]
fn main() -> ! {
    let p = stm32f103::Peripherals::take().unwrap();

    setup_clock(&p.RCC, &p.FLASH);
    p.RCC
        .apb2enr
        .write(|w| w.iopaen().set_bit().iopben().set_bit().iopcen().set_bit());
    p.RCC.apb1enr.write(|w| w.usben().set_bit());
    matrix::init(&p.GPIOA, &p.GPIOB);

    for _ in 0..80000 {
        cortex_m::asm::nop();
    }

    unsafe {
        pma::fill_with_zero();
    }
    let mut ctrl_buf = [0u8; 128];
    let config_descr_buf = unsafe {
        core::slice::from_raw_parts(
            (&CONFIG_DESCR as *const CompositeConfigDescriptor) as *const u8,
            core::mem::size_of::<CompositeConfigDescriptor>(),
        )
    };
    let mut kbd = USBKbd::new(p.USB, &DEVICE_DESCR, config_descr_buf, &mut ctrl_buf);
    kbd.setup();
    let mut stream = KeyStream::new();

    let mut clock_count: u32 = 0;
    let max = 0x00ffffff;

    p.STK.load_.write(|w| unsafe { w.reload().bits(max) });
    p.STK.val.write(|w| unsafe { w.current().bits(0) });
    p.STK
        .ctrl
        .write(|w| w.tickint().clear_bit().enable().set_bit());

    loop {
        kbd.usb_poll();

        if let ControlState::Idle { buf: _ } = kbd.ctrl_state {
            let elapsed = max - p.STK.val.read().bits();
            clock_count += elapsed;
            // Reset val
            p.STK.load_.write(|w| unsafe { w.reload().bits(max) });
            p.STK.val.write(|w| unsafe { w.current().bits(0) });
            p.STK
                .ctrl
                .write(|w| w.tickint().clear_bit().enable().set_bit());

            let mat = matrix::scan(&p.GPIOA, &p.GPIOB);
            let per = peer::scan();
            stream.push(&mat, &per, clock_count);
            stream.read(|k| {
                let mut buf = [0u8; 8];
                buf[0] = k[0]; // modifier
                buf[2] = k[1]; // keycode
                kbd.hid_send_keys(&buf)
            });
        }
    }
}
