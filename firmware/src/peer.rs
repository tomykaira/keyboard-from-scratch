use stm32l4xx_hal::gpio::gpioa::{PA10, PA9};
use stm32l4xx_hal::gpio::{Alternate, OpenDrain, Output, AF4};
use stm32l4xx_hal::i2c::{Error, I2c};
use stm32l4xx_hal::pac::I2C1;
use stm32l4xx_hal::prelude::*;

const SERIAL_SUB_BUFFER_LENGTH: usize = crate::direct_drive::SCAN_SIZE;
const PEER_OFFSET: u8 = 0x80;
pub(crate) const I2C_ADDRESS: u8 = 18;
type SclPin = PA9<Alternate<AF4, Output<OpenDrain>>>;
type SdaPin = PA10<Alternate<AF4, Output<OpenDrain>>>;

pub struct Peer {
    i2c: I2c<I2C1, (SclPin, SdaPin)>,
    pub serial_sub_buffer: [u8; SERIAL_SUB_BUFFER_LENGTH],
    pub error: Option<Error>,
}

impl Peer {
    pub fn new(i2c: I2c<I2C1, (SclPin, SdaPin)>) -> Peer {
        return Peer {
            i2c,
            serial_sub_buffer: [0u8; SERIAL_SUB_BUFFER_LENGTH],
            error: None,
        };
    }

    /// (no_error?, keys)
    pub fn read(&mut self) -> (bool, [u8; SERIAL_SUB_BUFFER_LENGTH]) {
        match self.i2c.read(I2C_ADDRESS, &mut self.serial_sub_buffer) {
            Err(err) => {
                self.error = Some(err);
                return (false, [0u8; SERIAL_SUB_BUFFER_LENGTH]);
            }
            Ok(_) => {}
        }
        let mut pos = [0u8; SERIAL_SUB_BUFFER_LENGTH];
        for i in 0..SERIAL_SUB_BUFFER_LENGTH {
            // Swap cols
            // 1, 2, 3 -> 4, 5, 6
            // 4, 5, 6 -> 3, 2, 1
            let row = self.serial_sub_buffer[i as usize] & 0xf0;
            let col = self.serial_sub_buffer[i as usize] & 0x0f;
            let adj_col = if col <= 3 { col + 3 } else { 7 - col };
            pos[i as usize] = (row | adj_col) | PEER_OFFSET;
        }
        (true, pos)
    }
}
