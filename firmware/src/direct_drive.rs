// Direct drive switches.
use embedded_hal::digital::v2::InputPin;
use stm32l4xx_hal::gpio;

pub const SCAN_SIZE: usize = 8;

type SW1_1 = gpio::gpiob::PB8<gpio::Input<gpio::PullUp>>;
type SW1_2 = gpio::gpiob::PB2<gpio::Input<gpio::PullUp>>;
type SW1_3 = gpio::gpiob::PB5<gpio::Input<gpio::PullUp>>;
type SW1_4 = gpio::gpioa::PA8<gpio::Input<gpio::PullUp>>;
type SW2_1 = gpio::gpiob::PB7<gpio::Input<gpio::PullUp>>;
type SW2_2 = gpio::gpiob::PB4<gpio::Input<gpio::PullUp>>;
type SW2_3 = gpio::gpiob::PB1<gpio::Input<gpio::PullUp>>;
type SW2_4 = gpio::gpiob::PB9<gpio::Input<gpio::PullUp>>;
type SW3_1 = gpio::gpiob::PB6<gpio::Input<gpio::PullUp>>;
type SW3_2 = gpio::gpiob::PB3<gpio::Input<gpio::PullUp>>;
type SW3_3 = gpio::gpiob::PB0<gpio::Input<gpio::PullUp>>;
type SW3_4 = gpio::gpioa::PA4<gpio::Input<gpio::PullUp>>;
type SW4_1 = gpio::gpiob::PB12<gpio::Input<gpio::PullUp>>;
type SW4_2 = gpio::gpiob::PB11<gpio::Input<gpio::PullUp>>;
type SW4_3 = gpio::gpiob::PB10<gpio::Input<gpio::PullUp>>;
type SW4_4 = gpio::gpioa::PA7<gpio::Input<gpio::PullUp>>;
type SW5_1 = gpio::gpiob::PB15<gpio::Input<gpio::PullUp>>;
type SW5_2 = gpio::gpiob::PB14<gpio::Input<gpio::PullUp>>;
type SW5_3 = gpio::gpiob::PB13<gpio::Input<gpio::PullUp>>;
type SW5_4 = gpio::gpioa::PA6<gpio::Input<gpio::PullUp>>;
type SW6_1 = gpio::gpioa::PA0<gpio::Input<gpio::PullUp>>;
type SW6_2 = gpio::gpioa::PA1<gpio::Input<gpio::PullUp>>;
type SW6_3 = gpio::gpioa::PA2<gpio::Input<gpio::PullUp>>;
type SW6_4 = gpio::gpioa::PA5<gpio::Input<gpio::PullUp>>;
type SW7_4 = gpio::gpioa::PA3<gpio::Input<gpio::PullUp>>;

pub struct Switches {
    sw1_1: SW1_1,
    sw1_2: SW1_2,
    sw1_3: SW1_3,
    sw1_4: SW1_4,
    sw2_1: SW2_1,
    sw2_2: SW2_2,
    sw2_3: SW2_3,
    sw2_4: SW2_4,
    sw3_1: SW3_1,
    sw3_2: SW3_2,
    sw3_3: SW3_3,
    sw3_4: SW3_4,
    sw4_1: SW4_1,
    sw4_2: SW4_2,
    sw4_3: SW4_3,
    sw4_4: SW4_4,
    sw5_1: SW5_1,
    sw5_2: SW5_2,
    sw5_3: SW5_3,
    sw5_4: SW5_4,
    sw6_1: SW6_1,
    sw6_2: SW6_2,
    sw6_3: SW6_3,
    sw6_4: SW6_4,
    sw7_4: SW7_4,
}

macro_rules! scan {
    ($sw:expr, $val:expr, $off:ident, $vec:ident) => {
        if $sw.is_low().unwrap() && $off < SCAN_SIZE {
            $vec[$off] = $val;
            $off += 1;
        }
    };
}

impl Switches {
    pub fn new(
        sw1_1: SW1_1,
        sw1_2: SW1_2,
        sw1_3: SW1_3,
        sw1_4: SW1_4,
        sw2_1: SW2_1,
        sw2_2: SW2_2,
        sw2_3: SW2_3,
        sw2_4: SW2_4,
        sw3_1: SW3_1,
        sw3_2: SW3_2,
        sw3_3: SW3_3,
        sw3_4: SW3_4,
        sw4_1: SW4_1,
        sw4_2: SW4_2,
        sw4_3: SW4_3,
        sw4_4: SW4_4,
        sw5_1: SW5_1,
        sw5_2: SW5_2,
        sw5_3: SW5_3,
        sw5_4: SW5_4,
        sw6_1: SW6_1,
        sw6_2: SW6_2,
        sw6_3: SW6_3,
        sw6_4: SW6_4,
        sw7_4: SW7_4,
    ) -> Switches {
        Switches {
            sw1_1,
            sw1_2,
            sw1_3,
            sw1_4,
            sw2_1,
            sw2_2,
            sw2_3,
            sw2_4,
            sw3_1,
            sw3_2,
            sw3_3,
            sw3_4,
            sw4_1,
            sw4_2,
            sw4_3,
            sw4_4,
            sw5_1,
            sw5_2,
            sw5_3,
            sw5_4,
            sw6_1,
            sw6_2,
            sw6_3,
            sw6_4,
            sw7_4,
        }
    }

    #[allow(unused_assignments)]
    pub fn scan(&mut self) -> [u8; SCAN_SIZE] {
        let mut off = 0;
        let mut vec = [0u8; SCAN_SIZE];
        scan!(self.sw1_1, 0x11, off, vec);
        scan!(self.sw1_2, 0x21, off, vec);
        scan!(self.sw1_3, 0x31, off, vec);
        scan!(self.sw1_4, 0x41, off, vec);

        scan!(self.sw2_1, 0x12, off, vec);
        scan!(self.sw2_2, 0x22, off, vec);
        scan!(self.sw2_3, 0x32, off, vec);
        scan!(self.sw2_4, 0x42, off, vec);

        scan!(self.sw3_1, 0x13, off, vec);
        scan!(self.sw3_2, 0x23, off, vec);
        scan!(self.sw3_3, 0x33, off, vec);
        scan!(self.sw3_4, 0x43, off, vec);

        scan!(self.sw4_1, 0x14, off, vec);
        scan!(self.sw4_2, 0x24, off, vec);
        scan!(self.sw4_3, 0x34, off, vec);
        scan!(self.sw4_4, 0x44, off, vec);

        scan!(self.sw5_1, 0x15, off, vec);
        scan!(self.sw5_2, 0x25, off, vec);
        scan!(self.sw5_3, 0x35, off, vec);
        scan!(self.sw5_4, 0x45, off, vec);

        scan!(self.sw6_1, 0x16, off, vec);
        scan!(self.sw6_2, 0x26, off, vec);
        scan!(self.sw6_3, 0x36, off, vec);
        scan!(self.sw6_4, 0x46, off, vec);

        scan!(self.sw7_4, 0x47, off, vec);
        return vec;
    }
}
