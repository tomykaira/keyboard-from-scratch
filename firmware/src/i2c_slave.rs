// ref: https://raw.githubusercontent.com/stm32-rs/stm32f0xx-hal/20f5520aef17aa08e752256ff6bd8b449788ed73/src/i2c_slave.rs
use core::mem;
use core::mem::MaybeUninit;
use stm32l4::stm32l4x2::I2C1;
pub use stm32l4xx_hal::i2c;
use stm32l4xx_hal::i2c::{SclPin, SdaPin};
use stm32l4xx_hal::rcc::{Clocks, APB1R1};
use stm32l4xx_hal::time::Hertz;

#[allow(unused_imports)]
#[cfg(feature = "semihosting")]
use cortex_m_semihosting::hprintln;
use stm32l4xx_hal::gpio;
use stm32l4xx_hal::prelude::OutputPin;

pub type Dbg1 = gpio::gpiob::PB2<gpio::Output<gpio::PushPull>>;
pub type Dbg2 = gpio::gpiob::PB1<gpio::Output<gpio::PushPull>>;
pub type Dbg3 = gpio::gpiob::PB0<gpio::Output<gpio::PushPull>>;

#[derive(PartialEq, Debug)]
pub enum TransferState {
    Idle,
    WaitingAddress,
    WaitingTxis,
    WaitingRxne,
}

const BUFFER_SIZE: usize = 32;

struct FixedSizeBuffer {
    buffer: [u8; BUFFER_SIZE],
    len: usize,
    index: usize,
}

impl FixedSizeBuffer {
    fn new() -> FixedSizeBuffer {
        FixedSizeBuffer {
            buffer: [0u8; BUFFER_SIZE],
            len: 0,
            index: 0,
        }
    }

    fn put(&mut self, b: u8) {
        self.buffer[self.index] = b;
        self.index += 1;
        self.len += 1;
    }

    fn get(&mut self) -> u8 {
        let v = self.buffer[self.index];
        self.index += 1;
        v
    }

    fn is_empty(&self) -> bool {
        return self.index >= self.len;
    }

    fn rewind(&mut self) {
        self.index = 0;
    }

    fn copy(&self) -> &[u8] {
        &self.buffer[0..self.len]
    }
}

pub struct I2CSlave<SCL, SDA> {
    i2c: I2C1,
    wbuf: FixedSizeBuffer,
    rbuf: FixedSizeBuffer,
    pub transfer_state: TransferState,
    pins: (SCL, SDA),
    address: u8,
    freq: Hertz,
    clocks: Clocks,
}

impl<SCLPIN, SDAPIN> I2CSlave<SCLPIN, SDAPIN> {
    pub fn i2c1(i2c: I2C1, pins: (SCLPIN, SDAPIN), address: u8, freq: Hertz, clocks: Clocks) -> Self
    where
        SCLPIN: SclPin<I2C1>,
        SDAPIN: SdaPin<I2C1>,
    {
        I2CSlave {
            i2c: i2c,
            wbuf: FixedSizeBuffer::new(),
            rbuf: FixedSizeBuffer::new(),
            transfer_state: TransferState::Idle,
            pins: pins,
            address,
            freq,
            clocks,
        }
    }
}

// It's s needed for the impls, but rustc doesn't recognize that
#[allow(dead_code)]
type I2cRegisterBlock = stm32l4xx_hal::pac::i2c1::RegisterBlock;

impl<SCL: SclPin<I2C1>, SDA: SdaPin<I2C1>> I2CSlave<SCL, SDA> {
    fn slave_initialization(&mut self, apb1: &mut APB1R1) {
        // Borrow I2C initialization from hal::i2c.
        // Free hal::i2c right after initialize.
        let i2c = mem::replace(&mut self.i2c, unsafe {
            MaybeUninit::uninit().assume_init()
        });
        let pins = mem::replace(&mut self.pins, unsafe {
            MaybeUninit::uninit().assume_init()
        });
        let hal_i2c = i2c::I2c::i2c1(i2c, pins, self.freq, self.clocks, apb1);
        let (f_i2c, f_pins) = hal_i2c.free();
        self.i2c = f_i2c;
        self.pins = f_pins;

        self.i2c.cr1.write(|w| w.gcen().enabled().pe().enabled());

        self.i2c.oar1.write(|w| w.oa1en().disabled());

        self.i2c
            .oar1
            .write(|w| w.oa1().bits((self.address as u16) << 1).oa1mode().bit7());

        self.i2c.oar1.write(|w| w.oa1en().enabled());
    }

    fn read(&self) -> u8 {
        self.i2c.rxdr.read().bits() as u8
    }

    fn write(&self, value: u8) {
        self.i2c.txdr.write(|w| w.txdata().bits(value));
    }

    pub fn receive_if_idle(&mut self, apb1: &mut APB1R1) {
        if self.transfer_state == TransferState::Idle {
            self.rbuf = FixedSizeBuffer::new();
            self.slave_initialization(apb1);
            self.transfer_state = TransferState::WaitingAddress;
        }
    }

    pub fn transmit(&mut self, apb1: &mut APB1R1, buffer: &[u8]) {
        self.wbuf = FixedSizeBuffer::new();
        for item in buffer {
            self.wbuf.put(*item);
        }
        self.wbuf.rewind();

        self.slave_initialization(apb1);
        self.transfer_state = TransferState::WaitingAddress;
    }

    pub fn poll(&mut self, dbg1: &mut Dbg1, dbg2: &mut Dbg2, dbg3: &mut Dbg3) {
        if self.i2c.cr1.read().pe().bit_is_set() && self.i2c.oar1.read().oa1en().bit_is_set() {
            dbg1.set_high().unwrap();
        } else {
            dbg1.set_low().unwrap();
        }
        if self.i2c.isr.read().berr().bit_is_set()
            || self.i2c.isr.read().nackf().bit_is_set()
            || self.i2c.isr.read().arlo().bit_is_set()
        {
            dbg2.set_high().unwrap();
        } else {
            dbg2.set_low().unwrap();
        }
        if self.i2c.isr.read().addr().is_match_() {
            dbg3.set_high().unwrap();
        } else {
            dbg3.set_low().unwrap();
        }

        match self.transfer_state {
            TransferState::Idle => {
                #[cfg(feature = "semihosting")]
                hprintln!("i").unwrap();
            } // no tasks
            TransferState::WaitingAddress => {
                #[cfg(feature = "semihosting")]
                hprintln!("a").unwrap();
                if self.i2c.isr.read().addr().is_match_() {
                    if self.i2c.isr.read().dir().is_write() {
                        self.transfer_state = TransferState::WaitingRxne;
                    } else {
                        self.transfer_state = TransferState::WaitingTxis;
                    }
                    self.i2c.icr.write(|w| w.addrcf().set_bit());
                }
            }
            TransferState::WaitingTxis => {
                #[cfg(feature = "semihosting")]
                hprintln!("t {}", self.wbuf.index).unwrap();
                if self.wbuf.is_empty() {
                    self.transfer_state = TransferState::Idle;
                } else if self.i2c.isr.read().stopf().is_stop() {
                    self.transfer_state = TransferState::Idle;
                } else if self.i2c.isr.read().txis().is_empty() {
                    let v = self.wbuf.get();
                    self.write(v);
                }
            }
            TransferState::WaitingRxne => {
                #[cfg(feature = "semihosting")]
                hprintln!("r").unwrap();
                if self.i2c.isr.read().rxne().is_empty() {
                    self.rbuf.put(self.read());
                }
                if self.i2c.isr.read().stopf().is_stop() {
                    self.transfer_state = TransferState::Idle;
                }
            }
        };
    }

    pub fn get_received_data(&mut self) -> &[u8] {
        if self.transfer_state == TransferState::Idle {
            self.rbuf.copy()
        } else {
            &[]
        }
    }

    pub fn release(self) -> (I2C1, (SCL, SDA)) {
        (self.i2c, self.pins)
    }
}
