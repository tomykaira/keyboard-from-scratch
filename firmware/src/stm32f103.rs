use crate::matrix::Matrix;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::gpio::gpiob::{PB6, PB7};
use stm32f1xx_hal::gpio::gpioc::PC13;
use stm32f1xx_hal::gpio::{Alternate, OpenDrain, Output, PushPull};
use stm32f1xx_hal::i2c::BlockingI2c;
use stm32f1xx_hal::i2c::Error;
use stm32f1xx_hal::pac::I2C1;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use stm32f1xx_hal::{i2c::Mode, prelude::*};
use usb_device::bus::UsbBusAllocator;

pub type Peripherals = stm32f1xx_hal::stm32::Peripherals;
pub type Clocks = stm32f1xx_hal::rcc::Clocks;
pub type LED = PC13<Output<PushPull>>;
pub type I2C = BlockingI2c<I2C1, (PB6<Alternate<OpenDrain>>, PB7<Alternate<OpenDrain>>)>;
pub type I2CError = nb::Error<Error>;
pub type UsbBusType = stm32f1xx_hal::usb::UsbBusType;

pub fn split(
    device: Peripherals,
) -> (
    Clocks,
    I2C,
    LED,
    UsbBusAllocator<UsbBus<Peripheral>>,
    Matrix,
) {
    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.constrain();

    let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = device.GPIOC.split(&mut rcc.apb2);

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    assert!(clocks.usbclk_valid());

    // I2C
    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);
    let mut afio = device.AFIO.constrain(&mut rcc.apb2);
    let i2c = BlockingI2c::i2c1(
        device.I2C1,
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

    // LED
    let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // USB
    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low().ok();

    let usb_dm = gpioa.pa11;
    let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);
    let usb = Peripheral {
        usb: device.USB,
        pin_dm: usb_dm,
        pin_dp: usb_dp,
    };
    let usb_bus = UsbBus::new(usb);

    // Matrix
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

    (clocks, i2c, led, usb_bus, matrix)
}
