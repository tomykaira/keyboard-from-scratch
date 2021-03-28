use embedded_hal::digital::v2::{InputPin, OutputPin};
use stm32l4xx_hal::gpio;

type R1 = gpio::gpiob::PB1<gpio::Output<gpio::PushPull>>;
type R2 = gpio::gpiob::PB5<gpio::Output<gpio::PushPull>>;
type R3 = gpio::gpiob::PB8<gpio::Output<gpio::PushPull>>;
type R4 = gpio::gpiob::PB9<gpio::Output<gpio::PushPull>>;

type C1 = gpio::gpiob::PB13<gpio::Input<gpio::PullUp>>;
type C2 = gpio::gpiob::PB14<gpio::Input<gpio::PullUp>>;
type C3 = gpio::gpiob::PB15<gpio::Input<gpio::PullUp>>;
type C4 = gpio::gpiob::PB2<gpio::Input<gpio::PullUp>>;
type C5 = gpio::gpiob::PB7<gpio::Input<gpio::PullUp>>;
type C6 = gpio::gpiob::PB4<gpio::Input<gpio::PullUp>>;

// Change pin assigns carefully, do not miss all of MODE, CNF, ODR, IDR.
#[derive(PartialEq, Eq)]
enum Row {
    R1, // B1
    R2, // B5
    R3, // B8
    R4, // B9
}

static ROWS: [Row; 4] = [Row::R1, Row::R2, Row::R3, Row::R4];

// Change pin assigns carefully, do not miss all of MODE, CNF, ODR, IDR.
#[derive(PartialEq, Eq)]
enum Col {
    C1, // B13
    C2, // B14
    C3, // B15
    C4, // A8
    C5, // A9
    C6, // A10
}

static COLS: [Col; 6] = [Col::C1, Col::C2, Col::C3, Col::C4, Col::C5, Col::C6];

pub struct Matrix {
    r1: R1,
    r2: R2,
    r3: R3,
    r4: R4,
    c1: C1,
    c2: C2,
    c3: C3,
    c4: C4,
    c5: C5,
    c6: C6,
}

impl Matrix {
    pub fn new(
        r1: R1,
        r2: R2,
        r3: R3,
        r4: R4,
        c1: C1,
        c2: C2,
        c3: C3,
        c4: C4,
        c5: C5,
        c6: C6,
    ) -> Matrix {
        Matrix {
            r1,
            r2,
            r3,
            r4,
            c1,
            c2,
            c3,
            c4,
            c5,
            c6,
        }
    }

    pub fn scan(&mut self) -> [u8; 8] {
        let mut off = 0;
        let mut vec = [0u8; 8];
        self.clear_row();
        for row in ROWS.iter() {
            self.set_row(row);
            for col in COLS.iter() {
                if self.read_col(col) && off < 8 {
                    vec[off] = encode(row, col);
                    off += 1;
                }
            }
            self.clear_row();
        }
        return vec;
    }

    // Active low
    fn set_row(&mut self, row: &Row) {
        if *row == Row::R1 {
            self.r1.set_low().unwrap();
        } else {
            self.r1.set_high().unwrap();
        }
        if *row == Row::R2 {
            self.r2.set_low().unwrap();
        } else {
            self.r2.set_high().unwrap();
        }
        if *row == Row::R3 {
            self.r3.set_low().unwrap();
        } else {
            self.r3.set_high().unwrap();
        }
        if *row == Row::R4 {
            self.r4.set_low().unwrap();
        } else {
            self.r4.set_high().unwrap();
        }
    }

    // Active low
    fn clear_row(&mut self) {
        self.r1.set_high().unwrap();
        self.r2.set_high().unwrap();
        self.r3.set_high().unwrap();
        self.r4.set_high().unwrap();
    }

    fn read_col(&self, col: &Col) -> bool {
        match col {
            Col::C1 => self.c1.is_low().unwrap(),
            Col::C2 => self.c2.is_low().unwrap(),
            Col::C3 => self.c3.is_low().unwrap(),
            Col::C4 => self.c4.is_low().unwrap(),
            Col::C5 => self.c5.is_low().unwrap(),
            Col::C6 => self.c6.is_low().unwrap(),
        }
    }
}

fn encode(row: &Row, col: &Col) -> u8 {
    let r = match row {
        Row::R1 => 1,
        Row::R2 => 2,
        Row::R3 => 3,
        Row::R4 => 4,
    };
    let c = match col {
        Col::C1 => 1,
        Col::C2 => 2,
        Col::C3 => 3,
        Col::C4 => 4,
        Col::C5 => 5,
        Col::C6 => 6,
    };
    return (r << 4) | c;
}
