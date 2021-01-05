use super::hid_keycodes as KC;

#[allow(unused_imports)]
use cortex_m_semihosting::hprintln;

pub struct KeyStream {
    /// Key event stream.
    events: [Event; 64],
    /// Read pointer in events.
    read_ptr: u8,
    /// Write pointer in events.
    write_ptr: u8,
    /// Positions currently on.
    on_pos: [bool; 256],
    /// Known last cnt value.
    last_cnt: u16,
    /// State to implement keyboard features.
    #[allow(dead_code)]
    state: FeatureState,
}

/// List of state variables used to implement our own features.
struct FeatureState {}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Action {
    NOP,
    DOWN,
    UP,
}

/// See `matrix` for encoding rule.
type Pos = u8;

/// Key event struct.
#[derive(Copy, Clone)]
#[allow(dead_code)]
struct Event {
    action: Action,
    pos: Pos,
    cnt: u16, // 1 = 1/128 sec
}

impl KeyStream {
    /// Initialize key stream.
    pub fn new() -> KeyStream {
        init_constants();
        KeyStream {
            events: [Event {
                action: Action::NOP,
                pos: 0,
                cnt: 0,
            }; 64],
            read_ptr: 0,
            write_ptr: 0,
            on_pos: [false; 256],
            last_cnt: 0,
            state: FeatureState {},
        }
    }

    /// Update key events by currently pressed key positions.
    pub fn push(&mut self, mat: &[Pos; 8], peer: &[Pos; 8], clk: u32) {
        let cnt = (clk >> 16) as u16;
        self.last_cnt = cnt;
        // skip 0, invalid pos.
        for i in 1u8..=0xffu8 {
            let on = is_on(mat, peer, i);
            let was_on = self.on_pos[i as usize];
            if was_on && !on {
                self.push_event(&Event {
                    action: Action::UP,
                    pos: i,
                    cnt,
                });
            }
            if !was_on && on {
                self.push_event(&Event {
                    action: Action::DOWN,
                    pos: i,
                    cnt,
                });
            }
            self.on_pos[i as usize] = on;
        }
    }

    fn push_event(&mut self, evt: &Event) {
        self.events[self.write_ptr as usize] = *evt;
        self.write_ptr += 1;
        if self.write_ptr >= 64 {
            self.write_ptr = 0;
        }
    }

    /// Return: `[modifier, key]`
    pub fn read<F>(&mut self, emit: F)
    where
        F: FnOnce([u8; 2]) -> Option<()>,
    {
        let mut key = [0u8; 2];
        unsafe {
            for i in 1..=0x46 {
                let k = SIMPLE_KEY_MAP[i];
                if self.on_pos[i] && k > 0 {
                    key[1] = k;
                }
            }
        }
        if self.on_pos[0x21] {
            key[0] |= KC::KBD_MODIFIER_LEFT_CTRL;
        }
        if self.on_pos[0x31] {
            key[0] |= KC::KBD_MODIFIER_LEFT_SHIFT;
        }
        if self.on_pos[0x42] {
            key[0] |= KC::KBD_MODIFIER_LEFT_ALT;
        }
        if self.on_pos[0x43] {
            key[0] |= KC::KBD_MODIFIER_LEFT_UI;
        }
        // TODO
        if let Some(_ev) = self.peek_event() {
            if let Some(_) = emit(key) {
                self.consume_event()
            }
        } else {
            emit(key);
        }
    }

    /// Read the first unprocessed event.
    fn peek_event(&mut self) -> Option<&Event> {
        if self.read_ptr == self.write_ptr {
            None
        } else {
            Some(&self.events[self.read_ptr as usize])
        }
    }

    /// Move read pointer forward.
    fn consume_event(&mut self) {
        if self.read_ptr != self.write_ptr {
            self.read_ptr += 1;
            if self.read_ptr >= 64 {
                self.read_ptr = 0;
            }
        }
    }
}

static mut SIMPLE_KEY_MAP: [u8; 256] = [0u8; 256];

fn init_constants() {
    unsafe {
        SIMPLE_KEY_MAP[0x11] = KC::KBD_TAB;
        SIMPLE_KEY_MAP[0x12] = KC::KBD_Q;
        SIMPLE_KEY_MAP[0x13] = KC::KBD_W;
        SIMPLE_KEY_MAP[0x14] = KC::KBD_E;
        SIMPLE_KEY_MAP[0x15] = KC::KBD_R;
        SIMPLE_KEY_MAP[0x16] = KC::KBD_T;
        // SIMPLE_KEY_MAP[0x21] = Ctrl;
        SIMPLE_KEY_MAP[0x22] = KC::KBD_A;
        SIMPLE_KEY_MAP[0x23] = KC::KBD_S;
        SIMPLE_KEY_MAP[0x24] = KC::KBD_D;
        SIMPLE_KEY_MAP[0x25] = KC::KBD_F;
        SIMPLE_KEY_MAP[0x26] = KC::KBD_G;
        // SIMPLE_KEY_MAP[0x31] = ShiftKC::KBD_;
        SIMPLE_KEY_MAP[0x32] = KC::KBD_Z;
        SIMPLE_KEY_MAP[0x33] = KC::KBD_X;
        SIMPLE_KEY_MAP[0x34] = KC::KBD_C;
        SIMPLE_KEY_MAP[0x35] = KC::KBD_V;
        SIMPLE_KEY_MAP[0x36] = KC::KBD_B;
        SIMPLE_KEY_MAP[0x41] = KC::KBD_TILDE;
        // SIMPLE_KEY_MAP[0x42] = Alt
        // SIMPLE_KEY_MAP[0x43] = UI
        // SIMPLE_KEY_MAP[0x44] = Mod1
        SIMPLE_KEY_MAP[0x45] = KC::KBD_SPACEBAR;
        // SIMPLE_KEY_MAP[0x46] = Mod2
    }
}

fn is_on(mat: &[Pos; 8], peer: &[Pos; 8], i: Pos) -> bool {
    for x in mat {
        if i == *x {
            return true;
        }
    }
    for x in peer {
        if i == *x {
            return true;
        }
    }
    return false;
}
