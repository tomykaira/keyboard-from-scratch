#![no_std]
#![no_main]
#![deny(warnings)]

extern crate panic_semihosting;
// extern crate panic_halt;

use cortex_m::{asm::delay, peripheral::DWT};
#[allow(unused_imports)]
use cortex_m_semihosting::hprintln;
use embedded_hal::digital::v2::OutputPin;
use rtic::cyccnt::{Instant, U32Ext as _};
use stm32f1xx_hal::i2c::BlockingI2c;
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use stm32f1xx_hal::{gpio, i2c::Mode, prelude::*};
use usb_device::bus;
use usb_device::prelude::*;

use hid::HIDClass;
use key_stream::ring_buffer::RingBuffer;
use key_stream::KeyStream;
use matrix::Matrix;
use peer::Peer;

mod hid;
mod matrix;
mod peer;

type LED = gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>;

const READ_PERIOD: u32 = 72_000; // CPU is 72MHz -> 1ms
const TRANSFORM_PERIOD: u32 = READ_PERIOD * 15; // 50ms
const SEND_PERIOD: u32 = READ_PERIOD; // 1ms

#[rtic::app(device = stm32f1xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        led: LED,

        usb_dev: UsbDevice<'static, UsbBusType>,
        hid: HIDClass<'static, UsbBusType>,
        stream: KeyStream,
        matrix: Matrix,
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

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = cx.device.GPIOC.split(&mut rcc.apb2);
        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .freeze(&mut flash.acr);

        assert!(clocks.usbclk_valid());

        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low().ok();
        delay(clocks.sysclk().0 / 100);

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

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
        let matrix = Matrix::new(
            gpiob.pb1.into_push_pull_output(&mut gpiob.crl),
            gpiob.pb5.into_push_pull_output(&mut gpiob.crl),
            gpiob.pb8.into_push_pull_output(&mut gpiob.crh),
            gpiob.pb9.into_push_pull_output(&mut gpiob.crh),
            gpiob.pb13.into_pull_up_input(&mut gpiob.crh),
            gpiob.pb14.into_pull_up_input(&mut gpiob.crh),
            gpiob.pb15.into_pull_up_input(&mut gpiob.crh),
            gpioa.pa8.into_pull_up_input(&mut gpioa.crh),
            gpioa.pa9.into_pull_up_input(&mut gpioa.crh),
            gpioa.pa10.into_pull_up_input(&mut gpioa.crh),
        );

        let mut afio = cx.device.AFIO.constrain(&mut rcc.apb2);
        let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
        let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);
        let i2c = BlockingI2c::i2c1(
            cx.device.I2C1,
            (scl, sda),
            &mut afio.mapr,
            Mode::Standard {
                frequency: 100_000.hz(),
            },
            clocks,
            &mut rcc.apb1,
            1000,
            10,
            1000,
            1000,
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
            matrix,
            peer,
            report_buffer: RingBuffer::new([0; 8]),
        }
    }

    #[task(schedule = [read_loop], resources = [led, stream, matrix, peer], priority = 1)]
    fn read_loop(mut cx: read_loop::Context) {
        cx.schedule
            .read_loop(Instant::now() + READ_PERIOD.cycles())
            .ok();

        let led = &mut cx.resources.led;
        let stream = &mut cx.resources.stream;
        let matrix = &mut cx.resources.matrix;
        let peer = &mut cx.resources.peer;

        let mat = matrix.scan();
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

    #[task(binds=USB_HP_CAN_TX, resources = [usb_dev, hid], priority = 2)]
    fn usb_tx(mut cx: usb_tx::Context) {
        usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.hid);
    }

    #[task(binds=USB_LP_CAN_RX0, resources = [usb_dev, hid], priority = 2)]
    fn usb_rx(mut cx: usb_rx::Context) {
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
