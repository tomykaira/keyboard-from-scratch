#![no_std]
#[deny(warnings)]
extern crate panic_semihosting;
// extern crate panic_halt;

use usb_device::bus;
use usb_device::prelude::*;

use hid::HIDClass;

pub mod app;
pub mod hid;
pub mod matrix;
pub mod peer;
#[cfg(feature = "stm32f072")]
pub mod stm32f072;
#[cfg(feature = "stm32f103")]
pub mod stm32f103;

#[cfg(feature = "stm32f072")]
use stm32f072 as stm;
#[cfg(feature = "stm32f103")]
use stm32f103 as stm;

pub fn usb_poll<B: bus::UsbBus>(
    usb_dev: &mut UsbDevice<'static, B>,
    hid: &mut HIDClass<'static, B>,
) {
    if !usb_dev.poll(&mut [hid]) {
        return;
    }
}
