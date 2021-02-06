use crate::matrix::Matrix;
use embedded_hal::digital::v2::OutputPin;
use stm32f0xx_hal::gpio::gpiob::{PB7, PB8};
use stm32f0xx_hal::gpio::gpioc::PC13;
use stm32f0xx_hal::gpio::{Alternate, Output, PushPull, AF1};
use stm32f0xx_hal::i2c::{Error, I2c};
use stm32f0xx_hal::pac::I2C1;
use stm32f0xx_hal::prelude::*;
use stm32f0xx_hal::usb::{Peripheral, UsbBus};
use usb_device::bus::UsbBusAllocator;

pub type Clocks = stm32f0xx_hal::rcc::Clocks;
pub type LED = PC13<Output<PushPull>>;
pub type I2C = I2c<I2C1, PB8<Alternate<AF1>>, PB7<Alternate<AF1>>>;
pub type I2CError = Error;
pub type UsbBusType = stm32f0xx_hal::usb::UsbBusType;

pub fn split(
    device: stm32f0xx_hal::stm32::Peripherals,
) -> (
    Clocks,
    I2C,
    LED,
    UsbBusAllocator<UsbBus<Peripheral>>,
    Matrix,
) {
    cortex_m::interrupt::free(move |cs| {
        let mut flash = device.FLASH;
        let mut rcc = device.RCC.configure().freeze(&mut flash);

        let gpioa = device.GPIOA.split(&mut rcc);
        let gpiob = device.GPIOB.split(&mut rcc);
        let gpioc = device.GPIOC.split(&mut rcc);

        // TODO: Clock config.
        let clocks = rcc.clocks;
        // .cfgr
        // .use_hse(8.mhz())
        // .sysclk(48.mhz())
        // .pclk1(24.mhz())
        // .freeze(&mut flash.acr);

        // I2C
        let sda = gpiob.pb7.into_alternate_af1(cs);
        let scl = gpiob.pb8.into_alternate_af1(cs);
        let i2c = I2c::i2c1(device.I2C1, (scl, sda), 100.khz(), &mut rcc);

        // LED
        let led = gpioc.pc13.into_push_pull_output(cs);

        // USB
        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(cs);
        usb_dp.set_low().ok();

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(cs);
        let usb = Peripheral {
            usb: device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };
        let usb_bus = UsbBus::new(usb);

        // Matrix
        let matrix = Matrix::new(
            gpiob.pb1.into_push_pull_output(cs),
            gpiob.pb5.into_push_pull_output(cs),
            gpiob.pb10.into_push_pull_output(cs),
            gpiob.pb9.into_push_pull_output(cs),
            gpiob.pb13.into_pull_up_input(cs),
            gpiob.pb14.into_pull_up_input(cs),
            gpiob.pb15.into_pull_up_input(cs),
            gpioa.pa8.into_pull_up_input(cs),
            gpioa.pa9.into_pull_up_input(cs),
            gpioa.pa10.into_pull_up_input(cs),
        );

        (clocks, i2c, led, usb_bus, matrix)
    })
}
