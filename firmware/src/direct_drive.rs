// Direct drive switches.
use embedded_hal::digital::v2::InputPin;
use stm32l4xx_hal::gpio;

type SW1 = gpio::gpiob::PB8<gpio::Input<gpio::PullUp>>;
type SW2 = gpio::gpiob::PB7<gpio::Input<gpio::PullUp>>;
type SW3 = gpio::gpiob::PB6<gpio::Input<gpio::PullUp>>;
type SW4 = gpio::gpiob::PB5<gpio::Input<gpio::PullUp>>;
type SW5 = gpio::gpiob::PB4<gpio::Input<gpio::PullUp>>;
type SW6 = gpio::gpiob::PB3<gpio::Input<gpio::PullUp>>;
type SW7 = gpio::gpiob::PB2<gpio::Input<gpio::PullUp>>;
type SW8 = gpio::gpiob::PB1<gpio::Input<gpio::PullUp>>;
type SW9 = gpio::gpiob::PB0<gpio::Input<gpio::PullUp>>;

pub struct Switches {
    sw1: SW1,
    sw2: SW2,
    sw3: SW3,
    sw4: SW4,
    sw5: SW5,
    sw6: SW6,
    sw7: SW7,
    sw8: SW8,
    sw9: SW9,
}

impl Switches {
    pub fn new(
        sw1: SW1,
        sw2: SW2,
        sw3: SW3,
        sw4: SW4,
        sw5: SW5,
        sw6: SW6,
        sw7: SW7,
        sw8: SW8,
        sw9: SW9,
    ) -> Switches {
        Switches {
            sw1,
            sw2,
            sw3,
            sw4,
            sw5,
            sw6,
            sw7,
            sw8,
            sw9,
        }
    }

    #[allow(unused_assignments)]
    pub fn scan(&mut self) -> [u8; 8] {
        let mut off = 0;
        let mut vec = [0u8; 8];
        if self.sw1.is_low().unwrap() {
            vec[off] = 0x11;
            off += 1;
        }
        if self.sw2.is_low().unwrap() {
            vec[off] = 0x12;
            off += 1;
        }
        if self.sw3.is_low().unwrap() {
            vec[off] = 0x13;
            off += 1;
        }
        if self.sw4.is_low().unwrap() {
            vec[off] = 0x21;
            off += 1;
        }
        if self.sw5.is_low().unwrap() {
            vec[off] = 0x22;
            off += 1;
        }
        if self.sw6.is_low().unwrap() {
            vec[off] = 0x23;
            off += 1;
        }
        if self.sw7.is_low().unwrap() {
            vec[off] = 0x31;
            off += 1;
        }
        if self.sw8.is_low().unwrap() {
            vec[off] = 0x32;
            off += 1;
        }
        if self.sw9.is_low().unwrap() {
            vec[off] = 0x33;
            off += 1;
        }
        return vec;
    }
}
