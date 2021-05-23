use stm32l4xx_hal::gpio::gpioa::{PA10, PA9};
use stm32l4xx_hal::gpio::{Alternate, OpenDrain, Output, AF4};
use stm32l4xx_hal::i2c::{Error, I2c};
use stm32l4xx_hal::pac::I2C1;
use stm32l4xx_hal::prelude::*;

const ROWS_PER_HAND: u8 = 1;
const SERIAL_SUB_BUFFER_LENGTH: u8 = 1;
// const I2C_ADDRESS: u8 = 0x32u8;
pub(crate) const I2C_ADDRESS: u8 = 0x19u8;
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
                for i in 0..ROWS_PER_HAND {
                    pos[i as usize] = self.serial_sub_buffer[i as usize];
                }
                (true, pos)
            }
        }
    }
}
