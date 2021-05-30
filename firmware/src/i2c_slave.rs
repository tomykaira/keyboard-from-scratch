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

pub type Dbg1 = gpio::gpiob::PB2<gpio::Output<gpio::PushPull>>;
pub type Dbg2 = gpio::gpiob::PB1<gpio::Output<gpio::PushPull>>;
pub type Dbg3 = gpio::gpiob::PB0<gpio::Output<gpio::PushPull>>;

#[derive(Debug)]
pub enum Error {
    /// Bus error
    Bus,
    /// Arbitration loss
    Arbitration,
    /// NACK
    Nack,
    /// Other transfer is ongoing
    Busy,
}

#[derive(Eq, PartialEq)]
enum State {
    /// No ongoing transfer
    Idle,
    /// Receiving
    BusyRx,
    /// Transmitting
    BusyTx,
}

pub struct I2CSlave<SCL, SDA> {
    i2c: I2C1,
    pins: (SCL, SDA),
    address: u8,
    state: State,
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
            pins: pins,
            address,
            state: State::Idle,
            freq,
            clocks,
        }
    }
}

// It's s needed for the impls, but rustc doesn't recognize that
#[allow(dead_code)]
type I2cRegisterBlock = stm32l4xx_hal::pac::i2c1::RegisterBlock;

macro_rules! busy_wait {
    ($i2c:expr, $flag:ident, $variant:ident) => {
        loop {
            let isr = $i2c.isr.read();

            if isr.$flag().$variant() {
                break;
            } else if isr.berr().is_error() {
                $i2c.icr.write(|w| w.berrcf().set_bit());
                $i2c.cr2.write(|w| w.nack().set_bit());
                return Err(Error::Bus);
            } else if isr.arlo().is_lost() {
                $i2c.icr.write(|w| w.arlocf().set_bit());
                $i2c.cr2.write(|w| w.nack().set_bit());
                return Err(Error::Arbitration);
            } else if isr.nackf().bit_is_set() {
                $i2c.icr.write(|w| w.stopcf().set_bit().nackcf().set_bit());
                // flush_txdr!($i2c);
                $i2c.cr2.write(|w| w.nack().set_bit());
                return Err(Error::Nack);
            } else {
                // try again
            }
        }
    };
}

impl<SCL: SclPin<I2C1>, SDA: SdaPin<I2C1>> I2CSlave<SCL, SDA> {
    pub fn slave_initialization(&mut self, apb1: &mut APB1R1) {
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

        self.i2c.cr1.write(|w| w.pe().disabled());
        self.i2c.oar1.write(|w| w.oa1en().disabled());

        // Configure address
        self.i2c.oar1.write(|w| {
            w.oa1()
                .bits((self.address as u16) << 1)
                .oa1mode()
                .bit7()
                .oa1en()
                .enabled()
        });

        // Set default values.
        self.i2c
            .cr2
            .write(|w| w.autoend().automatic().nack().set_bit());

        // NOSTRETCH is mandatory for my hardware setup.
        self.i2c
            .cr1
            .write(|w| w.nostretch().enabled().anfoff().disabled());

        self.i2c.cr1.write(|w| w.pe().enabled());
    }

    fn read(&self) -> u8 {
        self.i2c.rxdr.read().bits() as u8
    }

    fn write(&self, value: u8) {
        self.i2c.txdr.write(|w| w.txdata().bits(value));
    }

    pub fn receive(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // if self.state != State::Idle {
        //     return Err(Error::Busy);
        // }

        assert!(buffer.len() > 0);

        // TODO: timeout
        // let tickstart = HAL_GetTick();

        self.state = State::BusyRx;

        // Enable Address Acknowledge
        self.i2c.cr2.write(|w| w.nack().clear_bit());

        // Wait until ADDR flag is set
        // TODO: timeout
        busy_wait!(self.i2c, addr, is_match_);
        // On timeout:
        // {
        //     /* Disable Address Acknowledge */
        //     hi2c->Instance->CR2 |= I2C_CR2_NACK;
        //     return HAL_ERROR;
        // }

        /* Clear ADDR flag */
        self.i2c.icr.write(|w| w.addrcf().set_bit());

        /* Wait until DIR flag is reset Receiver mode */
        busy_wait!(self.i2c, dir, bit_is_clear);
        // On timeout:
        // {
        //     /* Disable Address Acknowledge */
        //     hi2c->Instance->CR2 |= I2C_CR2_NACK;
        //     return HAL_ERROR;
        // }

        let mut off = 0;
        while off < buffer.len() {
            /* Wait until RXNE flag is set */
            busy_wait!(self.i2c, rxne, is_not_empty);
            /* Read data from RXDR */
            buffer[off] = self.read();
            off += 1;
        }

        /* Wait until STOP flag is set */
        busy_wait!(self.i2c, stopf, bit_is_set);
        /* Clear STOP flag */
        self.i2c.icr.write(|w| w.stopcf().set_bit());

        /* Wait until BUSY flag is reset */
        busy_wait!(self.i2c, busy, bit_is_clear);

        /* Disable Address Acknowledge */
        self.i2c.cr2.write(|w| w.nack().set_bit());

        self.state = State::Idle;
        return Ok(());
    }

    pub fn transmit(&mut self, buffer: &[u8]) -> Result<(), Error> {
        // if self.state != State::Idle {
        //     return Err(Error::Busy);
        // }

        assert!(buffer.len() > 0);

        // TODO: Init tickstart for timeout management
        // tickstart = HAL_GetTick();

        self.state = State::BusyTx;

        /* Enable Address Acknowledge */
        self.i2c.cr2.write(|w| w.nack().clear_bit());

        // Wait until ADDR flag is set
        // TODO: timeout
        busy_wait!(self.i2c, addr, is_match_);
        // On timeout:
        // {
        //     /* Disable Address Acknowledge */
        //     hi2c->Instance->CR2 |= I2C_CR2_NACK;
        //     return HAL_ERROR;
        // }

        /* Clear ADDR flag */
        self.i2c.icr.write(|w| w.addrcf().set_bit());

        /* Wait until DIR flag is reset Receiver mode */
        busy_wait!(self.i2c, dir, bit_is_set);
        // On timeout:
        // {
        //     /* Disable Address Acknowledge */
        //     hi2c->Instance->CR2 |= I2C_CR2_NACK;
        //     return HAL_ERROR;
        // }

        let mut off = 0;
        while off < buffer.len() {
            // Wait until we are allowed to send data
            // (START has been ACKed or last byte when
            // through)
            busy_wait!(self.i2c, txis, is_empty);

            // Put byte on the wire
            self.write(buffer[off]);
            off += 1;
        }

        /* Wait until STOP flag is set */
        busy_wait!(self.i2c, stopf, bit_is_set);
        /* Clear STOP flag */
        self.i2c.icr.write(|w| w.stopcf().set_bit());

        /* Wait until BUSY flag is reset */
        busy_wait!(self.i2c, busy, bit_is_clear);

        /* Disable Address Acknowledge */
        self.i2c.cr2.write(|w| w.nack().set_bit());

        self.state = State::Idle;
        Ok(())
    }

    pub fn release(self) -> (I2C1, (SCL, SDA)) {
        (self.i2c, self.pins)
    }
}
