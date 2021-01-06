use cortex_m::asm::nop;
use stm32f1::stm32f103::{GPIOB, I2C1, RCC};

#[allow(unused_imports)]
use cortex_m_semihosting::hprintln;

const SLAVE_ADDRESS: u8 = 25u8;
const I2C_LOOP_TIMEOUT: u16 = 0x1000u16; // tekitou
const ROWS_PER_HAND: u8 = 5; // TODO: 4
const COLS_PER_HAND: u8 = 7; // TODO: 6

pub fn init(rcc: &RCC, i2c: &I2C1, gpiob: &GPIOB) {
    // Alternate Open-drain, 2MHz
    gpiob.crl.modify(|_, w| {
        w.cnf6()
            .bits(0b11)
            .mode6()
            .bits(0b10)
            .cnf7()
            .bits(0b11)
            .mode7()
            .bits(0b10)
    });

    // Init I2C
    rcc.apb1enr.modify(|_, w| w.i2c1en().set_bit());
    rcc.apb1rstr.modify(|_, w| w.i2c1rst().set_bit());
    // busywait
    for _ in 0..8000 {
        cortex_m::asm::nop();
    }
    rcc.apb1rstr.modify(|_, w| w.i2c1rst().clear_bit());

    i2c_master_init(i2c);

    hprintln!("in {}", i2c.cr1.read().pe().bit_is_set());
}

fn i2c_delay(i2c: &I2C1) {
    let mut lim: u16 = 0;
    while i2c.sr2.read().busy().bit_is_set() && lim < I2C_LOOP_TIMEOUT {
        lim += 1;
    }
}

fn i2c_master_init(i2c: &I2C1) {
    // Make sure peripheral is disable
    i2c.cr1.reset();
    i2c.cr2.reset();

    // • Program the peripheral input clock in I2C_CR2 Register in order to generate correct timings
    i2c.cr2.write(|w| unsafe { w.freq().bits(4u8) });
    // • Configure the clock control registers
    i2c.ccr
        .write(|w| unsafe { w.f_s().clear_bit().duty().clear_bit().ccr().bits(0x14) });
    // • Configure the rise time register
    i2c.trise.write(|w| w.trise().bits(0x5));
    // • Program the I2C_CR1 register to enable the peripheral
    i2c.cr1.modify(|_, w| w.pe().enabled());
}

fn i2c_master_reset(i2c: &I2C1) {
    i2c.cr1.write(|w| w.pe().set_bit().swrst().set_bit());
    i2c.cr1.reset();
    i2c_master_init(i2c);
}

fn i2c_master_start(i2c: &I2C1, is_write: bool) {
    // • Set the START bit in the I2C_CR1 register to generate a Start condition
    i2c.cr1.modify(|_, w| w.start().set_bit());
    // while !i2c.sr1.read().sb().bit_is_set() || !i2c.sr1.read().berr().bit_is_set() {
    //     nop();
    // }
    for _ in 0..80000 {
        cortex_m::asm::nop();
    }
    hprintln!(
        "q {} {} {}",
        i2c.sr1.read().sb().bit_is_set(),
        i2c.sr1.read().berr().bit_is_set(),
        i2c.sr1.read().timeout().bit_is_set(),
    )
    .unwrap();
    let addr = ((SLAVE_ADDRESS << 1) | (if is_write { 0 } else { 1 }));
    i2c.dr.write(|w| w.dr().bits(addr));
    while !i2c.sr1.read().addr().bit_is_set() {
        nop();
    }
}

fn i2c_master_stop(i2c: &I2C1) {
    i2c.cr1
        .modify(|_, w| w.start().clear_bit().stop().set_bit());
    // Reset by hardware when stopped.
    while !i2c.sr1.read().tx_e().bit_is_clear() {
        nop();
    }
}

fn i2c_master_write(i2c: &I2C1, data: u8) {
    while !i2c.sr1.read().tx_e().bit_is_set() {
        nop();
    }
    i2c.dr.write(|w| w.dr().bits(data));
    while !i2c.sr1.read().btf().bit_is_set() {
        nop();
    }
}

fn i2c_master_read(i2c: &I2C1) -> u8 {
    i2c.cr1.modify(|_, w| w.ack().set_bit());
    while !i2c.sr1.read().rx_ne().bit_is_set() {
        nop();
    }
    return i2c.dr.read().dr().bits();
}

pub fn scan(i2c: &I2C1) -> [u8; 8] {
    i2c_master_reset(i2c);
    hprintln!("a").unwrap();
    i2c_master_start(i2c, true);
    hprintln!("b").unwrap();
    i2c_master_write(i2c, 0x00u8);
    hprintln!("c").unwrap();
    i2c_master_start(i2c, false);
    hprintln!("d").unwrap();
    let mut mat = [0u8; ROWS_PER_HAND as usize];
    for i in 0..ROWS_PER_HAND {
        mat[i as usize] = i2c_master_read(i2c);
    }
    hprintln!("e").unwrap();
    i2c_master_stop(i2c);

    hprintln!("v{} {} {} {} {}", mat[0], mat[1], mat[2], mat[3], mat[4]).unwrap();

    let mut v = [0u8; 8];
    let mut idx = 0;
    for i in 0..ROWS_PER_HAND {
        for j in 0..COLS_PER_HAND {
            if mat[i as usize] & (1 << j) != 0 {
                v[idx] = ((i + 5) << 4) | (j + 1);
                idx += 1;
            }
        }
    }
    return v;
}
