#![no_std]
#![no_main]
#![deny(warnings)]
extern crate firmware;

extern crate panic_semihosting;
// extern crate panic_halt;

use usb_device::bus;
use usb_device::prelude::*;

use firmware::hid::HIDClass;

use cortex_m::asm::delay;
use firmware::app::App;
use firmware::stm32f103 as stm;
use rtic::cyccnt::{Instant, U32Ext};
use rtic::export::DWT;

const READ_PERIOD: u32 = 72_000; // CPU is 72MHz -> 1ms
const TRANSFORM_PERIOD: u32 = READ_PERIOD * 15; // 50ms
const SEND_PERIOD: u32 = READ_PERIOD; // 1ms

#[rtic::app(device = stm32f1xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        app: App,
        hid: HIDClass<'static, stm::UsbBusType>,
        usb_dev: UsbDevice<'static, stm::UsbBusType>,
    }

    #[init(schedule = [read_loop, transform_loop, send_loop])]
    fn init(mut cx: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<stm::UsbBusType>> = None;

        cx.core.DCB.enable_trace();
        DWT::unlock();
        cx.core.DWT.enable_cycle_counter();

        let (clocks, i2c, led, usb, matrix) = stm::split(cx.device);
        *USB_BUS = Some(usb);

        delay(clocks.sysclk().0 / 100);

        let hid = HIDClass::new(USB_BUS.as_ref().unwrap());

        let usb_dev = UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0xc410, 0x0000))
            .manufacturer("tomykaira")
            .product("FLAT7")
            .serial_number("TEST")
            .device_class(0)
            .build();

        let app = App::new(i2c, led, matrix);

        cx.schedule.read_loop(cx.start + READ_PERIOD.cycles()).ok();
        cx.schedule
            .transform_loop(cx.start + TRANSFORM_PERIOD.cycles())
            .ok();
        cx.schedule.send_loop(cx.start + SEND_PERIOD.cycles()).ok();

        init::LateResources { app, usb_dev, hid }
    }

    #[task(schedule = [read_loop], resources = [app], priority = 1)]
    fn read_loop(mut cx: read_loop::Context) {
        cx.schedule
            .read_loop(Instant::now() + READ_PERIOD.cycles())
            .ok();

        let app = &mut cx.resources.app;
        app.read();
    }

    #[task(schedule = [transform_loop], resources = [app], priority = 1)]
    fn transform_loop(mut cx: transform_loop::Context) {
        cx.schedule
            .transform_loop(Instant::now() + TRANSFORM_PERIOD.cycles())
            .ok();
        let app = &mut cx.resources.app;
        app.transform();
    }

    #[task(schedule = [send_loop], resources = [app, hid], priority = 1)]
    fn send_loop(mut cx: send_loop::Context) {
        cx.schedule
            .send_loop(Instant::now() + SEND_PERIOD.cycles())
            .ok();
        let app = &mut cx.resources.app;
        cx.resources.hid.lock(|hid| app.send(hid));
    }

    #[task(binds=USB_HP_CAN_TX, resources = [usb_dev, hid], priority = 2)]
    fn usb_tx(mut cx: usb_tx::Context) {
        firmware::usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.hid);
    }

    #[task(binds=USB_LP_CAN_RX0, resources = [usb_dev, hid], priority = 2)]
    fn usb_rx(mut cx: usb_rx::Context) {
        firmware::usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.hid);
    }

    extern "C" {
        fn EXTI0();
    }
};
