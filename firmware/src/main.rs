#![no_std]
#![no_main]
// #![deny(warnings)]

#[cfg(not(feature = "semihosting"))]
extern crate panic_halt;
#[cfg(feature = "semihosting")]
extern crate panic_semihosting;

use cortex_m::peripheral::DWT;
#[allow(unused_imports)]
#[cfg(feature = "semihosting")]
use cortex_m_semihosting::hprintln;
use embedded_hal::digital::v2::OutputPin;
use rtic::cyccnt::{Instant, U32Ext as _};
use stm32l4xx_hal::i2c::I2c;
use stm32l4xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use stm32l4xx_hal::{gpio, prelude::*, stm32};
use usb_device::bus;
use usb_device::prelude::*;

use direct_drive::Switches;
use hid::HIDClass;
use key_stream::ring_buffer::RingBuffer;
use key_stream::KeyStream;
use peer::Peer;
use stm32l4xx_hal::rcc::{PllConfig, PllDivider};

mod direct_drive;
mod hid;
// mod matrix;
mod peer;

type LED = gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>;

// Do not change CLOCK while using STM32L412.
const CLOCK: u32 = 48; // MHz
const READ_PERIOD: u32 = CLOCK * 1000; // about 1ms
const TRANSFORM_PERIOD: u32 = READ_PERIOD * 15; // about 15ms
const SEND_PERIOD: u32 = READ_PERIOD; // 1ms

#[rtic::app(device = stm32l4xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        led: LED,

        usb_dev: UsbDevice<'static, UsbBusType>,
        hid: HIDClass<'static, UsbBusType>,
        stream: KeyStream,
        switches: Switches,
        peer: Peer,
        report_buffer: RingBuffer<[u8; 8]>,
    }

    #[init(schedule = [read_loop, transform_loop, send_loop])]
    fn init(mut cx: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBusType>> = None;

        cx.core.DCB.enable_trace();
        DWT::unlock();
        cx.core.DWT.enable_cycle_counter();

        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.ahb2);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.ahb2);
        let mut gpioc = cx.device.GPIOC.split(&mut rcc.ahb2);
        let led = gpioc
            .pc13
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
        let mut pwr = cx.device.PWR.constrain(&mut rcc.apb1r1);

        // TODO: Is this clock correct for usb?
        let clocks = rcc
            .cfgr
            .hsi48(true)
            // HSI = 16MHz, Sys clock = 48MHz, plln >= 8 ,plldiv = 2 -> mul = 18, div = 3
            .sysclk_with_pll(CLOCK.mhz(), PllConfig::new(3, 18 as u8, PllDivider::Div2))
            .pclk1(28.mhz())
            .pclk2(28.mhz())
            .freeze(&mut flash.acr, &mut pwr);

        enable_crs();
        enable_usb_pwr();

        let usb_dm = gpioa.pa11.into_af10(&mut gpioa.moder, &mut gpioa.afrh);
        let usb_dp = gpioa.pa12.into_af10(&mut gpioa.moder, &mut gpioa.afrh);

        let usb = Peripheral {
            usb: cx.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        *USB_BUS = Some(UsbBus::new(usb));

        let hid = HIDClass::new(USB_BUS.as_ref().unwrap());

        let usb_dev = UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0xc410, 0x0000))
            .manufacturer("tomykaira")
            .product("FLAT7")
            .serial_number("TEST")
            .device_class(0)
            .build();

        let stream = KeyStream::new();
        let switches = Switches::new(
            gpiob
                .pb8
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb7
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb6
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb5
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb4
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb3
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb2
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb1
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb0
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
        );

        let mut scl = gpioa
            .pa9
            .into_open_drain_output(&mut gpioa.moder, &mut gpioa.otyper);
        scl.internal_pull_up(&mut gpioa.pupdr, true);
        let scl = scl.into_af4(&mut gpioa.moder, &mut gpioa.afrh);

        let mut sda = gpioa
            .pa10
            .into_open_drain_output(&mut gpioa.moder, &mut gpioa.otyper);
        sda.internal_pull_up(&mut gpioa.pupdr, true);
        let sda = sda.into_af4(&mut gpioa.moder, &mut gpioa.afrh);
        let i2c = I2c::i2c1(
            cx.device.I2C1,
            (scl, sda),
            100_000.hz(),
            clocks,
            &mut rcc.apb1r1,
        );

        let peer = Peer::new(i2c);

        cx.schedule.read_loop(cx.start + READ_PERIOD.cycles()).ok();
        cx.schedule
            .transform_loop(cx.start + TRANSFORM_PERIOD.cycles())
            .ok();
        cx.schedule.send_loop(cx.start + SEND_PERIOD.cycles()).ok();

        init::LateResources {
            led,

            usb_dev,
            hid,
            stream,
            switches,
            peer,
            report_buffer: RingBuffer::new([0; 8]),
        }
    }

    #[task(schedule = [read_loop], resources = [led, stream, switches, peer], priority = 1)]
    fn read_loop(mut cx: read_loop::Context) {
        cx.schedule
            .read_loop(Instant::now() + READ_PERIOD.cycles())
            .ok();

        let led = &mut cx.resources.led;
        let stream = &mut cx.resources.stream;
        let switches = &mut cx.resources.switches;
        let peer = &mut cx.resources.peer;

        let mat = switches.scan();
        let (ok, per) = peer.read();
        if ok {
            led.set_high().unwrap();
        } else {
            led.set_low().unwrap();
            // match peer.error {
            //     None => {}
            //     Some(nb::Error::WouldBlock) => debug(hid, KBD_A),
            //     Some(nb::Error::Other(i2c::Error::Acknowledge)) => debug(hid, KBD_B),
            //     Some(nb::Error::Other(i2c::Error::Arbitration)) => debug(hid, KBD_D),
            //     Some(nb::Error::Other(i2c::Error::Bus)) => debug(hid, KBD_E),
            //     Some(nb::Error::Other(i2c::Error::Overrun)) => debug(hid, KBD_F),
            //     Some(nb::Error::Other(i2c::Error::_Extensible)) => debug(hid, KBD_X),
            // }
        }
        stream.push(&mat, &per, DWT::get_cycle_count());
    }

    #[task(schedule = [transform_loop], resources = [stream, report_buffer], priority = 1)]
    fn transform_loop(mut cx: transform_loop::Context) {
        cx.schedule
            .transform_loop(Instant::now() + TRANSFORM_PERIOD.cycles())
            .ok();

        let stream = &mut cx.resources.stream;
        let report_buffer = &mut cx.resources.report_buffer;

        stream.read(DWT::get_cycle_count(), |k| {
            report_buffer.push(&k);
        });
    }

    #[task(schedule = [send_loop], resources = [hid, report_buffer], priority = 1)]
    fn send_loop(mut cx: send_loop::Context) {
        cx.schedule
            .send_loop(Instant::now() + SEND_PERIOD.cycles())
            .ok();

        let hid = &mut cx.resources.hid;
        let report_buffer = &mut cx.resources.report_buffer;

        if let Some(k) = report_buffer.peek(0) {
            match hid.lock(|h| h.write(&k)) {
                Err(UsbError::WouldBlock) => (),
                Err(UsbError::BufferOverflow) => panic!("BufferOverflow"),
                Err(_) => panic!("Undocumented usb error"),
                Ok(_) => report_buffer.consume(),
            }
        }
    }

    #[task(binds=USB, resources = [usb_dev, hid], priority = 2)]
    fn usb_tx(mut cx: usb_tx::Context) {
        usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.hid);
    }

    extern "C" {
        fn EXTI0();
        fn I2C1();
    }
};

fn usb_poll<B: bus::UsbBus>(usb_dev: &mut UsbDevice<'static, B>, hid: &mut HIDClass<'static, B>) {
    if !usb_dev.poll(&mut [hid]) {
        return;
    }
}

fn enable_crs() {
    let rcc = unsafe { &(*stm32::RCC::ptr()) };
    rcc.apb1enr1.modify(|_, w| w.crsen().set_bit());
    let crs = unsafe { &(*stm32::CRS::ptr()) };
    // Initialize clock recovery
    // Set autotrim enabled.
    crs.cr.modify(|_, w| w.autotrimen().set_bit());
    // Enable CR
    crs.cr.modify(|_, w| w.cen().set_bit());
}

/// Enables VddUSB power supply
fn enable_usb_pwr() {
    // Enable PWR peripheral
    let rcc = unsafe { &(*stm32::RCC::ptr()) };
    rcc.apb1enr1.modify(|_, w| w.pwren().set_bit());

    // Enable VddUSB
    let pwr = unsafe { &*stm32::PWR::ptr() };
    pwr.cr2.modify(|_, w| w.usv().set_bit());
}
