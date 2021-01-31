const BUFFER_SIZE: usize = 64;

pub struct RingBuffer<T: Copy> {
    /// Buffer, stream of T.
    buf: [T; BUFFER_SIZE],
    /// Read pointer in buffer.
    read_ptr: usize,
    /// Write pointer in buffer.
    write_ptr: usize,
}

impl<T: Copy> RingBuffer<T> {
    pub fn new(t: T) -> RingBuffer<T> {
        RingBuffer {
            buf: [t; BUFFER_SIZE],
            read_ptr: 0,
            write_ptr: 0,
        }
    }

    /// Push item to buffer
    pub fn push(&mut self, item: &T) {
        self.buf[self.write_ptr] = *item;
        self.write_ptr += 1;
        if self.write_ptr >= BUFFER_SIZE {
            self.write_ptr = 0;
        }
    }

    /// Read the first unprocessed item.
    pub fn peek(&self, offset: usize) -> Option<T> {
        assert!(offset < BUFFER_SIZE);
        let read_pos = self.read_ptr + offset;
        let write_pos = if self.write_ptr < self.read_ptr {
            self.write_ptr + BUFFER_SIZE
        } else {
            self.write_ptr
        };
        if read_pos >= write_pos {
            None
        } else {
            Some(self.buf[(read_pos % BUFFER_SIZE)])
        }
    }

    /// Move read pointer forward.
    pub fn consume(&mut self) {
        if self.read_ptr != self.write_ptr {
            self.read_ptr += 1;
            if self.read_ptr >= BUFFER_SIZE {
                self.read_ptr = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ring_buffer::{RingBuffer, BUFFER_SIZE};

    #[test]
    fn test_push_peek() {
        let mut buf = RingBuffer::new(0u8);
        buf.push(&1);
        assert_eq!(buf.peek(0), Some(1));
        assert_eq!(buf.peek(1), None);
        buf.consume();
        assert_eq!(buf.peek(0), None);
    }

    #[test]
    fn test_push_peek_multi() {
        let mut buf = RingBuffer::new(0u8);
        buf.push(&1);
        buf.push(&2);
        buf.push(&3);
        assert_eq!(buf.peek(0), Some(1));
        assert_eq!(buf.peek(1), Some(2));
        assert_eq!(buf.peek(2), Some(3));
        assert_eq!(buf.peek(3), None);
        buf.consume();
        assert_eq!(buf.peek(0), Some(2));
    }

    #[test]
    fn test_go_around() {
        let mut buf = RingBuffer::new(0u8);
        for i in 0..BUFFER_SIZE - 2 {
            buf.push(&(i as u8));
            buf.consume();
        }
        buf.push(&1);
        buf.push(&2);
        buf.push(&3);
        assert_eq!(buf.peek(0), Some(1));
        assert_eq!(buf.peek(1), Some(2));
        assert_eq!(buf.peek(2), Some(3));
        assert_eq!(buf.peek(3), None);
        buf.consume();
        assert_eq!(buf.peek(0), Some(2));
    }
}
