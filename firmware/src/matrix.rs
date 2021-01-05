use super::gpio;
use stm32f1::stm32f103::{GPIOA, GPIOB};

#[allow(unused_imports)]
use cortex_m_semihosting::hprintln;

// Change pin assigns carefully, do not miss all of MODE, CNF, ODR, IDR.
#[derive(PartialEq, Eq)]
#[allow(dead_code)]
enum Row {
    R1, // B1
    R2, // B5
    R3, // B8
    R4, // B9
}

static ROWS: [Row; 4] = [Row::R1, Row::R2, Row::R3, Row::R4];

// Change pin assigns carefully, do not miss all of MODE, CNF, ODR, IDR.
#[derive(PartialEq, Eq)]
#[allow(dead_code)]
enum Col {
    C1, // B13
    C2, // B14
    C3, // B15
    C4, // A8
    C5, // A9
    C6, // A10
}

static COLS: [Col; 6] = [Col::C1, Col::C2, Col::C3, Col::C4, Col::C5, Col::C6];

pub fn init(gpioa: &GPIOA, gpiob: &GPIOB) {
    gpiob.crh.write(|w| {
        w.mode8()
            .bits(gpio::Mode::Output2MHz.bits())
            .cnf8()
            .bits(gpio::OutputCnf::Pushpull.bits())
            .mode9()
            .bits(gpio::Mode::Output2MHz.bits())
            .cnf9()
            .bits(gpio::OutputCnf::Pushpull.bits())
            .mode13()
            .bits(gpio::Mode::Input.bits())
            .cnf13()
            .bits(gpio::InputCnf::PullUpdown.bits())
            .mode14()
            .bits(gpio::Mode::Input.bits())
            .cnf14()
            .bits(gpio::InputCnf::PullUpdown.bits())
            .mode15()
            .bits(gpio::Mode::Input.bits())
            .cnf15()
            .bits(gpio::InputCnf::PullUpdown.bits())
    });
    gpioa.crh.write(|w| {
        w.mode10()
            .bits(gpio::Mode::Input.bits())
            .cnf10()
            .bits(gpio::InputCnf::PullUpdown.bits())
            .mode8()
            .bits(gpio::Mode::Input.bits())
            .cnf8()
            .bits(gpio::InputCnf::PullUpdown.bits())
            .mode9()
            .bits(gpio::Mode::Input.bits())
            .cnf9()
            .bits(gpio::InputCnf::PullUpdown.bits())
    });
    gpiob.crl.write(|w| {
        w.mode1()
            .bits(gpio::Mode::Output2MHz.bits())
            .cnf1()
            .bits(gpio::OutputCnf::Pushpull.bits())
            .mode5()
            .bits(gpio::Mode::Output2MHz.bits())
            .cnf5()
            .bits(gpio::OutputCnf::Pushpull.bits())
    });
    gpioa
        .odr
        .write(|w| w.odr8().set_bit().odr9().set_bit().odr10().set_bit());
    gpiob
        .odr
        .write(|w| w.odr13().set_bit().odr14().set_bit().odr15().set_bit());
}

pub fn scan(gpioa: &GPIOA, gpiob: &GPIOB) -> [u8; 8] {
    let mut off = 0;
    let mut vec = [0u8; 8];
    for row in ROWS.iter() {
        set_row(row, gpiob);
        for col in COLS.iter() {
            if read_col(col, gpioa, gpiob) {
                vec[off] = encode(row, col);
                off += 1;
            }
        }
    }
    return vec;
}

fn set_row(row: &Row, gpiob: &GPIOB) {
    for _ in 0..1000 {
        gpiob.odr.write(|w| {
            let w = w
                .odr13()
                .set_bit()
                .odr14()
                .set_bit()
                .odr15()
                .set_bit()
                .odr1()
                .set_bit()
                .odr5()
                .set_bit()
                .odr8()
                .set_bit()
                .odr9()
                .set_bit();
            match row {
                Row::R1 => w.odr1().clear_bit(),
                Row::R2 => w.odr5().clear_bit(),
                Row::R3 => w.odr8().clear_bit(),
                Row::R4 => w.odr9().clear_bit(),
            }
        })
    }
}

fn read_col(col: &Col, gpioa: &GPIOA, gpiob: &GPIOB) -> bool {
    match col {
        Col::C1 => gpiob.idr.read().idr13().is_low(),
        Col::C2 => gpiob.idr.read().idr14().is_low(),
        Col::C3 => gpiob.idr.read().idr15().is_low(),
        Col::C4 => gpioa.idr.read().idr8().is_low(),
        Col::C5 => gpioa.idr.read().idr9().is_low(),
        Col::C6 => gpioa.idr.read().idr10().is_low(),
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
