use stm32f1::stm32f103::{DWT, GPIOB, RCC};

use crate::gpio;
use cortex_m::interrupt;

const ROWS_PER_HAND: u8 = 4; // TODO: 4
const SERIAL_DELAY_US: u32 = 27;
const DELAY_1US_CYCLES: u32 = 72; // 72MHz
const SERIAL_SUB_BUFFER_LENGTH: u8 = ROWS_PER_HAND;
const START_BIT_TIMEOUT_CYCLE: u32 = DELAY_1US_CYCLES * 1000;

pub(crate) struct Peer<'a> {
    gpiob: &'a GPIOB,
    dwt: &'a DWT,
    serial_sub_buffer: [u8; SERIAL_SUB_BUFFER_LENGTH as usize],
    pub error: ErrorPoint,
}

#[derive(Debug, PartialEq)]
pub enum ErrorPoint {
    None,
    UartNoStart,
    UartBadParity,
    UartBadStop,
}

impl Peer<'_> {
    pub fn new<'a>(rcc: &RCC, dwt: &'a DWT, gpiob: &'a GPIOB) -> Peer<'a> {
        rcc.apb2enr.modify(|_, w| w.iopben().set_bit());
        return Peer {
            gpiob,
            dwt,
            serial_sub_buffer: [0u8; SERIAL_SUB_BUFFER_LENGTH as usize],
            error: ErrorPoint::None,
        };
    }

    pub fn init(&self) {
        self.serial_output();
        self.serial_high();
    }

    pub fn scan(&mut self) -> (bool, [u8; 8]) {
        self.serial_update_buffers();
        if self.error != ErrorPoint::None {
            return (false, [0u8; 8]);
        }
        let mut pos = [0u8; 8];
        let mut off = 0;
        for i in 0..ROWS_PER_HAND {
            let value = self.serial_sub_buffer[i as usize];
            for j in 0..8 {
                let bit = j << 1;
                if value & bit != 0 {
                    pos[off] = encode(i, bit);
                    off += 1;
                }
            }
        }
        (true, pos)
    }

    /*
    void serial_sub_init(void) {
      serial_input();

      // Enable INT0
      EIMSK |= _BV(INT0);
      // Trigger on falling edge of INT0
      EICRA &= ~(_BV(ISC00) | _BV(ISC01));
    }

     */

    #[inline]
    fn serial_output(&self) {
        self.gpiob.crl.modify(|_, w| {
            w.mode7()
                .bits(gpio::Mode::Output2MHz.bits())
                .cnf7()
                .bits(gpio::OutputCnf::Pushpull.bits())
        });
    }

    #[inline]
    fn serial_input(&self) {
        self.gpiob.crl.modify(|_, w| {
            w.mode7()
                .bits(gpio::Mode::Input.bits())
                .cnf7()
                .bits(gpio::InputCnf::PullUpdown.bits())
        });
    }

    #[inline]
    fn serial_high(&self) {
        self.gpiob.odr.modify(|_, w| w.odr7().set_bit());
    }

    #[inline]
    fn serial_low(&self) {
        self.gpiob.odr.modify(|_, w| w.odr7().clear_bit());
    }

    #[inline]
    fn serial_read_pin(&self) -> bool {
        self.gpiob.idr.read().idr7().bit_is_set()
    }

    fn wait_to(&self, cnt: u32) {
        while self.dwt.cyccnt.read() < cnt {}
    }

    fn read_uart(&mut self) -> (ErrorPoint, u8) {
        let mut byte = 0u8;
        let mut err = ErrorPoint::UartNoStart;

        // busy wait for falling of start bit
        for _ in 0..START_BIT_TIMEOUT_CYCLE {
            if !self.serial_read_pin() {
                err = ErrorPoint::None;
                break;
            }
        }
        if err != ErrorPoint::None {
            return (err, 0);
        }
        // Data will start after 1d, wait 1.5d to read mid of data
        let first_read =
            self.dwt.cyccnt.read() + DELAY_1US_CYCLES * (SERIAL_DELAY_US + SERIAL_DELAY_US / 2);
        for i in 0..8 {
            self.wait_to(first_read + i * SERIAL_DELAY_US * DELAY_1US_CYCLES);
            let b = b2u(self.serial_read_pin());
            byte = (byte << 1) | b;
        }
        // todo verify parity
        self.wait_to(first_read + 8 * SERIAL_DELAY_US * DELAY_1US_CYCLES);
        let p = self.serial_read_pin();
        // todo verify stop
        self.wait_to(first_read + 9 * SERIAL_DELAY_US * DELAY_1US_CYCLES);
        let s = self.serial_read_pin();
        if p != odd_parity(byte) {
            err = ErrorPoint::UartBadParity;
        }
        if !s && err == ErrorPoint::None {
            err = ErrorPoint::UartBadStop;
        }
        return (err, byte);
    }

    /// Copies the serial_sub_buffer to the MAIN and sends the
    /// serial_main_buffer to the sub.
    ///
    /// Returns:
    /// true => no error
    /// false => SUB did not respond
    fn serial_update_buffers(&mut self) {
        self.error = ErrorPoint::None;
        // this code is very time dependent, so we need to disable interrupts
        interrupt::disable();

        // signal to the SUB that we want to start a transaction
        // timing: 0
        self.serial_output();
        self.serial_low();
        self.wait_to(self.dwt.cyccnt.read() + SERIAL_DELAY_US * DELAY_1US_CYCLES / 2);
        // timing: 1d
        self.serial_high();

        // At timing 2d, sub starts sending UART bytes
        // Wait for input with pull-up.
        self.serial_input();
        self.serial_high();

        for i in 0..SERIAL_SUB_BUFFER_LENGTH {
            let (e, b) = self.read_uart();
            self.error = e;
            self.serial_sub_buffer[i as usize] = b;
        }

        // always, release the line when not in use
        self.serial_output();
        self.serial_high();

        unsafe {
            interrupt::enable();
        }

        // self.delay += 4;
    }
}

#[inline]
fn b2u(b: bool) -> u8 {
    b as u8
}

fn odd_parity(byte: u8) -> bool {
    let mut p = true;
    for i in 0..8 {
        if byte & (1 << i) != 0 {
            p = !p
        }
    }
    p
}

fn encode(row: u8, col: u8) -> u8 {
    let c = match col {
        32 => 1,
        2 => 2,
        4 => 3,
        8 => 4,
        16 => 5,
        1 => 6,
        _ => 0,
    };
    // Set top bit to indicate peer.
    return 0x80u8 | (row << 4) | c;
}
