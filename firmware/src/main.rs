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
use rtic::cyccnt::{Instant, U32Ext as _};
use stm32l4xx_hal::i2c::I2c;
use stm32l4xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use stm32l4xx_hal::{prelude::*, stm32};
use usb_device::bus;
use usb_device::prelude::*;

use crate::i2c_slave::I2CSlave;
use direct_drive::Switches;
use hid::HIDClass;
use key_stream::ring_buffer::RingBuffer;
use key_stream::KeyStream;
use peer::Peer;
use stm32l4xx_hal::gpio::{Alternate, OpenDrain, Output, PA10, PA9};
use stm32l4xx_hal::rcc::{PllConfig, PllDivider, APB1R1};

mod direct_drive;
mod hid;
// mod matrix;
mod i2c_slave;
mod peer;
mod reset;

// Do not change CLOCK while using STM32L412.
const CLOCK: u32 = 48; // MHz
const READ_PERIOD: u32 = CLOCK * 1000; // about 1ms
const TRANSFORM_PERIOD: u32 = READ_PERIOD * 15; // about 15ms
const SEND_PERIOD: u32 = READ_PERIOD; // 1ms
const SLAVE_TIMEOUT: u32 = 1000; // pseudo cycles.

#[rtic::app(device = stm32l4xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        usb_dev: Option<UsbDevice<'static, UsbBusType>>,
        hid: Option<HIDClass<'static, UsbBusType>>,
        stream: KeyStream,
        switches: Switches,
        peer: Option<Peer>,
        report_buffer: RingBuffer<[u8; 8]>,
        slave: Option<
            I2CSlave<
                PA9<Alternate<stm32l4xx_hal::gpio::AF4, Output<OpenDrain>>>,
                PA10<Alternate<stm32l4xx_hal::gpio::AF4, Output<OpenDrain>>>,
            >,
        >,
        apb1: APB1R1,
    }

    #[init(schedule = [read_loop, transform_loop, send_loop, slave_loop])]
    fn init(mut cx: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBusType>> = None;

        cx.core.DCB.enable_trace();
        DWT::unlock();
        cx.core.DWT.enable_cycle_counter();

        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.ahb2);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.ahb2);
        let mut pwr = cx.device.PWR.constrain(&mut rcc.apb1r1);

        let clocks = rcc
            .cfgr
            .hsi48(true)
            // HSI = 16MHz, Sys clock = 48MHz, plln >= 8 ,plldiv = 2 -> mul = 18, div = 3
            .sysclk_with_pll(CLOCK.mhz(), PllConfig::new(3, 18 as u8, PllDivider::Div2))
            .pclk1(28.mhz())
            .pclk2(28.mhz())
            .freeze(&mut flash.acr, &mut pwr);

        enable_crs();

        let stream = KeyStream::new();
        let switches = Switches::new(
            gpiob
                .pb8
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb2
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb5
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpioa
                .pa8
                .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr),
            gpiob
                .pb7
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb4
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb1
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb9
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb6
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb3
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb0
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpioa
                .pa4
                .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr),
            gpiob
                .pb12
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb11
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb10
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpioa
                .pa7
                .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr),
            gpiob
                .pb15
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb14
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb13
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpioa
                .pa6
                .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa
                .pa0
                .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa
                .pa1
                .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa
                .pa2
                .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa
                .pa5
                .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa
                .pa3
                .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr),
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

        if cfg!(feature = "host") {
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

            let usb_dev =
                UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0xc410, 0x0000))
                    .manufacturer("tomykaira")
                    .product("FLAT7")
                    .serial_number("TEST")
                    .device_class(0)
                    .build();
            let i2c = I2c::i2c1(
                cx.device.I2C1,
                (scl, sda),
                100_000.hz(),
                clocks,
                &mut rcc.apb1r1,
            );

            cx.schedule.read_loop(cx.start + READ_PERIOD.cycles()).ok();
            cx.schedule
                .transform_loop(cx.start + TRANSFORM_PERIOD.cycles())
                .ok();
            cx.schedule.send_loop(cx.start + SEND_PERIOD.cycles()).ok();

            init::LateResources {
                usb_dev: Some(usb_dev),
                hid: Some(hid),
                stream,
                switches,
                peer: Some(Peer::new(i2c)),
                report_buffer: RingBuffer::new([0; 8]),
                slave: None,
                apb1: rcc.apb1r1,
            }
        } else {
            let mut slave = I2CSlave::i2c1(
                cx.device.I2C1,
                (scl, sda),
                peer::I2C_ADDRESS,
                100_000.hz(),
                clocks,
            );
            slave.slave_initialization(&mut rcc.apb1r1);

            cx.schedule.slave_loop(cx.start + 1.cycles()).ok();

            init::LateResources {
                usb_dev: None,
                hid: None,
                stream,
                switches,
                peer: None,
                report_buffer: RingBuffer::new([0; 8]),
                slave: Some(slave),
                apb1: rcc.apb1r1,
            }
        }
    }

    #[task(schedule = [slave_loop], resources = [slave, switches], priority = 1)]
    fn slave_loop(mut cx: slave_loop::Context) {
        cx.schedule.slave_loop(Instant::now() + 1.cycles()).ok();

        let slave = &mut cx.resources.slave;
        let switches = &mut cx.resources.switches;

        if let Some(ref mut slave) = slave {
            let mat = switches.scan();
            match slave.transmit(&mat, SLAVE_TIMEOUT) {
                Ok(()) => {}
                Err(e) => {
                    let _ = e;
                    #[cfg(feature = "semihosting")]
                    hprintln!("slave_error ?:", e);
                }
            }
        }
    }

    #[task(schedule = [read_loop], resources = [stream, switches, peer], priority = 1)]
    fn read_loop(mut cx: read_loop::Context) {
        cx.schedule
            .read_loop(Instant::now() + READ_PERIOD.cycles())
            .ok();

        let stream = &mut cx.resources.stream;
        let switches = &mut cx.resources.switches;
        let peer = &mut cx.resources.peer;

        let mat = switches.scan();
        match peer {
            Some(p) => {
                let (ok, per) = p.read();
                if ok {
                    #[cfg(feature = "semihosting")]
                    hprintln!("h");
                } else {
                    #[cfg(feature = "semihosting")]
                    hprintln!("v");
                }
                stream.push(&mat, &per, DWT::get_cycle_count());
            }
            None => (),
        }
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

        if stream.requests_reset() {
            unsafe {
                reset::reset();
            }
        }
    }

    #[task(schedule = [send_loop], resources = [hid, report_buffer], priority = 1)]
    fn send_loop(mut cx: send_loop::Context) {
        cx.schedule
            .send_loop(Instant::now() + SEND_PERIOD.cycles())
            .ok();

        let hid = &mut cx.resources.hid;
        let report_buffer = &mut cx.resources.report_buffer;

        if let Some(k) = report_buffer.peek(0) {
            match hid.lock(|h| h.as_mut().unwrap().write(&k)) {
                Err(UsbError::WouldBlock) => (),
                Err(UsbError::BufferOverflow) => panic!("BufferOverflow"),
                Err(_) => panic!("Undocumented usb error"),
                Ok(_) => report_buffer.consume(),
            }
        }
    }

    #[task(binds=USB, resources = [usb_dev, hid], priority = 2)]
    fn usb_tx(cx: usb_tx::Context) {
        usb_poll(
            &mut cx.resources.usb_dev.as_mut().unwrap(),
            &mut cx.resources.hid.as_mut().unwrap(),
        );
    }

    extern "C" {
        fn EXTI0();
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
