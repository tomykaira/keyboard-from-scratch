use stm32l4xx_hal::gpio::gpioa::{PA10, PA9};
use stm32l4xx_hal::gpio::{Alternate, OpenDrain, Output, AF4};
use stm32l4xx_hal::i2c::{Error, I2c};
use stm32l4xx_hal::pac::I2C1;
use stm32l4xx_hal::prelude::*;

const ROWS_PER_HAND: u8 = 4;
const SERIAL_SUB_BUFFER_LENGTH: u8 = 5;
// const I2C_ADDRESS: u8 = 0x32u8;
const I2C_ADDRESS: u8 = 0x19u8;
type SclPin = PA9<Alternate<AF4, Output<OpenDrain>>>;
type SdaPin = PA10<Alternate<AF4, Output<OpenDrain>>>;

pub struct Peer {
    i2c: I2c<I2C1, (SclPin, SdaPin)>,
    pub serial_sub_buffer: [u8; SERIAL_SUB_BUFFER_LENGTH as usize],
    pub error: Option<Error>,
}

impl Peer {
    pub fn new(i2c: I2c<I2C1, (SclPin, SdaPin)>) -> Peer {
        return Peer {
            i2c,
            serial_sub_buffer: [0u8; SERIAL_SUB_BUFFER_LENGTH as usize],
            error: None,
        };
    }

    /// (no_error?, keys)
    pub fn read(&mut self) -> (bool, [u8; 8]) {
        match self
            .i2c
            .write_read(I2C_ADDRESS, &[0x0u8], &mut self.serial_sub_buffer)
        {
            Err(err) => {
                self.error = Some(err);
                (false, [0u8; 8])
            }
            Ok(_) => {
                let mut pos = [0u8; 8];
                let mut off = 0;
                for i in 0..ROWS_PER_HAND {
                    let value = self.serial_sub_buffer[i as usize];
                    for j in 0..8 {
                        let bit = 1 << j;
                        if (value & bit) != 0 && off < 8 {
                            pos[off] = encode(i, bit);
                            off += 1;
                        }
                    }
                }
                (true, pos)
            }
        }
    }
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
    return 0x80u8 | ((row + 1) << 4) | c;
}
