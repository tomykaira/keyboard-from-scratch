use crate::hid::HIDClass;
use crate::matrix::Matrix;
use crate::peer::Peer;
use crate::stm;
use embedded_hal::digital::v2::OutputPin;
use key_stream::ring_buffer::RingBuffer;
use key_stream::KeyStream;
use rtic::export::DWT;
use usb_device::bus::UsbBus;
use usb_device::UsbError;

pub struct App {
    led: stm::LED,

    stream: KeyStream,
    matrix: Matrix,
    peer: Peer,
    report_buffer: RingBuffer<[u8; 8]>,
}

impl App {
    pub fn new(i2c: stm::I2C, led: stm::LED, matrix: Matrix) -> App {
        let stream = KeyStream::new();

        let peer = Peer::new(i2c);

        App {
            led,
            stream,
            matrix,
            peer,
            report_buffer: RingBuffer::new([0; 8]),
        }
    }

    pub fn read(&mut self) {
        let mat = self.matrix.scan();
        let (ok, per) = self.peer.read();
        if ok {
            self.led.set_high().unwrap();
        } else {
            self.led.set_low().unwrap();
            // match peer.error {
            //     None => {}
            //     Some(nb::Error::WouldBlock) => debug(hid, KBD_A),
            //     Some(nb::Error::Other(i2c::Error::Acknowledge)) => debug(hid, KBD_B),
            //     Some(nb::Error::Other(i2c::Error::Arbitration)) => debug(hid, KBD_D),
            //     Some(nb::Error::Other(i2c::Error::Bus)) => debug(hid, KBD_E),
            //     Some(nb::Error::Other(i2c::Error::Overrun)) => debug(hid, KBD_F),
            //     Some(nb::Error::Other(i2c::Error::_Extensible)) => debug(hid, KBD_X),
            // }
        }
        self.stream.push(&mat, &per, DWT::get_cycle_count());
    }

    pub fn transform(&mut self) {
        let report_buffer = &mut self.report_buffer;
        self.stream.read(DWT::get_cycle_count(), |k| {
            report_buffer.push(&k);
        });
    }

    pub fn send<B: UsbBus>(&mut self, hid: &mut HIDClass<'static, B>) {
        if let Some(k) = self.report_buffer.peek(0) {
            match hid.write(&k) {
                Err(UsbError::WouldBlock) => (),
                Err(UsbError::BufferOverflow) => panic!("BufferOverflow"),
                Err(_) => panic!("Undocumented usb error"),
                Ok(_) => self.report_buffer.consume(),
            }
        }
    }
}
