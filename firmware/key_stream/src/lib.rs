#![no_std]
#![deny(warnings)]

#[cfg(test)]
#[macro_use]
extern crate std;

mod hid_keycodes;
pub mod ring_buffer;

use crate::hid_keycodes as KC;
use crate::ring_buffer::RingBuffer;

const REPORT_SLOTS: usize = 6;

pub struct KeyStream {
    /// Key event stream.
    events: RingBuffer<Event>,
    /// Positions currently on.
    on_pos: [bool; 256],
    /// State to implement keyboard features.
    state: FeatureState,
}

/// List of state variables used to implement our own features.
struct FeatureState {
    mod_key_code: u8,
    mod1: bool,
    mod2: bool,
    pressed: [Kc; REPORT_SLOTS],
}

impl FeatureState {
    fn new() -> FeatureState {
        FeatureState {
            mod_key_code: 0,
            mod1: false,
            mod2: false,
            pressed: [0u8; REPORT_SLOTS],
        }
    }

    fn press(&mut self, kc: Kc) {
        if kc == KC::KBD_NONE {
            return;
        }
        for i in 0..REPORT_SLOTS {
            if self.pressed[i] == kc {
                return;
            }
            if self.pressed[i] == KC::KBD_NONE {
                self.pressed[i] = kc;
                return;
            }
        }
        // no more slots.
    }

    fn release(&mut self, kc: Kc) {
        if kc == KC::KBD_NONE {
            return;
        }
        for i in 0..REPORT_SLOTS {
            if self.pressed[i] == kc {
                for j in i..REPORT_SLOTS - 1 {
                    self.pressed[j] = self.pressed[j + 1];
                }
                return;
            }
            if self.pressed[i] == KC::KBD_NONE {
                return;
            }
        }
    }

    fn make_key_report(&self) -> [u8; 8] {
        let mut key = [0u8; REPORT_SLOTS + 2];
        key[0] = self.mod_key_code;
        for i in 0..REPORT_SLOTS {
            key[i + 2] = self.pressed[i];
        }
        return key;
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Action {
    DOWN,
    UP,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ModifierKey {
    CTRL1,
    SHIFT1,
    MOD1,
    ALT1,
    UI1,
    MOD2,
    SHIFT2,
}

impl ModifierKey {
    fn pos(&self) -> Pos {
        match *self {
            ModifierKey::CTRL1 => 0x21,
            ModifierKey::SHIFT1 => 0x31,
            ModifierKey::MOD1 => 0x42,
            ModifierKey::ALT1 => 0x43,
            ModifierKey::UI1 => 0x44,
            ModifierKey::MOD2 => 0x46,
            ModifierKey::SHIFT2 => 0xc1,
        }
    }

    fn code(&self) -> u8 {
        match *self {
            ModifierKey::CTRL1 => KC::KBD_MODIFIER_LEFT_CTRL,
            ModifierKey::SHIFT1 => KC::KBD_MODIFIER_LEFT_SHIFT,
            ModifierKey::MOD1 => 0,
            ModifierKey::ALT1 => KC::KBD_MODIFIER_LEFT_ALT,
            ModifierKey::UI1 => KC::KBD_MODIFIER_LEFT_UI,
            ModifierKey::MOD2 => 0,
            ModifierKey::SHIFT2 => KC::KBD_MODIFIER_LEFT_SHIFT,
        }
    }

    fn modifier(pos: Pos) -> Option<ModifierKey> {
        if pos == ModifierKey::CTRL1.pos() {
            Some(ModifierKey::CTRL1)
        } else if pos == ModifierKey::SHIFT1.pos() {
            Some(ModifierKey::SHIFT1)
        } else if pos == ModifierKey::MOD1.pos() {
            Some(ModifierKey::MOD1)
        } else if pos == ModifierKey::ALT1.pos() {
            Some(ModifierKey::ALT1)
        } else if pos == ModifierKey::UI1.pos() {
            Some(ModifierKey::UI1)
        } else if pos == ModifierKey::MOD2.pos() {
            Some(ModifierKey::MOD2)
        } else if pos == ModifierKey::SHIFT2.pos() {
            Some(ModifierKey::SHIFT2)
        } else {
            None
        }
    }
}

/// See `matrix` for encoding rule.
type Pos = u8;
/// See `hid_keycodes` for mapping.
type Kc = u8;

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
            events: RingBuffer::new(Event {
                action: Action::UP,
                pos: 0,
                cnt: 0,
            }),
            on_pos: [false; 256],
            state: FeatureState::new(),
        }
    }

    /// Update key events by currently pressed key positions.
    pub fn push(&mut self, mat: &[Pos; 8], peer: &[Pos; 8], clk: u32) {
        let cnt = (clk >> 16) as u16;
        for i in &VALID_KEY_LIST {
            let on = is_on(mat, peer, *i);
            let was_on = self.on_pos[*i as usize];
            if was_on && !on {
                self.push_event(&Event {
                    action: Action::UP,
                    pos: *i,
                    cnt,
                });
            }
            if !was_on && on {
                self.push_event(&Event {
                    action: Action::DOWN,
                    pos: *i,
                    cnt,
                });
            }
            self.on_pos[*i as usize] = on;
        }
    }

    fn push_event(&mut self, evt: &Event) {
        self.events.push(evt)
    }

    /// Return: `[modifier, key]`
    pub fn read<F>(&mut self, mut emit: F)
    where
        F: FnMut([u8; 8]) -> (),
    {
        let mut executed = false;
        while let Some(ev) = self.peek_event(0) {
            self.proc_event(&ev, &mut emit);
            executed = true;
        }
        if !executed {
            emit(self.state.make_key_report());
        }
    }

    fn proc_event<F>(&mut self, ev: &Event, mut emit: F)
    where
        F: FnMut([u8; 8]) -> (),
    {
        if ev.pos == 0 {
            // skip pos = 0, empty event.
            self.consume_event();
            return;
        }

        match ev.action {
            Action::DOWN => match ModifierKey::modifier(ev.pos) {
                None => unsafe {
                    if let Some(kc) = self.process_combo_keys(ev.pos) {
                        self.state.press(kc);
                        emit(self.state.make_key_report());
                        self.consume_event(); // skip one more event.
                    } else {
                        let map = if self.state.mod1 {
                            &MOD1_KEY_MAP
                        } else if self.state.mod2 {
                            &MOD2_KEY_MAP
                        } else {
                            &SIMPLE_KEY_MAP
                        };
                        let k = map[ev.pos as usize];
                        if k > 0 {
                            self.state.press(k);
                        }
                        emit(self.state.make_key_report());
                    }
                },
                Some(ModifierKey::MOD1) => self.state.mod1 = true,
                Some(ModifierKey::MOD2) => self.state.mod2 = true,
                Some(other) => {
                    self.state.mod_key_code = self.state.mod_key_code | other.code();
                    emit(self.state.make_key_report());
                }
            },
            Action::UP => match ModifierKey::modifier(ev.pos) {
                None => {
                    // refer all maps to allow pressing / releasing mod while char keys are on
                    self.release_related_keys(ev.pos);
                }
                Some(ModifierKey::MOD1) => self.state.mod1 = false,
                Some(ModifierKey::MOD2) => self.state.mod2 = false,
                Some(other) => self.state.mod_key_code = self.state.mod_key_code & (!other.code()),
            },
        }
        self.consume_event();
    }

    fn release_related_keys(&mut self, pos: Pos) {
        unsafe {
            let k = MOD1_KEY_MAP[pos as usize];
            if k > 0 {
                self.state.release(k);
            }
            let k = MOD2_KEY_MAP[pos as usize];
            if k > 0 {
                self.state.release(k);
            }
            let k = SIMPLE_KEY_MAP[pos as usize];
            if k > 0 {
                self.state.release(k);
            }
            for (k1, k2, kc) in COMBO_KEYS.iter() {
                if pos == *k1 || pos == *k2 {
                    self.state.release(*kc);
                }
            }
        }
    }

    fn process_combo_keys(&self, pos: Pos) -> Option<Kc> {
        if let Some((other, kc)) = expect_combo_key(pos) {
            if let Some(next) = self.peek_event(1) {
                if next.pos == other {
                    return Some(kc);
                }
            }
        }
        return None;
    }

    /// Read the first unprocessed event.
    /// offset: 0 to read head. 1 to read head + 1.
    fn peek_event(&self, offset: usize) -> Option<Event> {
        self.events.peek(offset)
    }

    /// Move read pointer forward.
    fn consume_event(&mut self) {
        self.events.consume()
    }
}

static VALID_KEY_LIST: [u8; 48] = [
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x31, 0x32, 0x33, 0x34,
    0x35, 0x36, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0xa1, 0xa2,
    0xa3, 0xa4, 0xa5, 0xa6, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6,
];

static mut SIMPLE_KEY_MAP: [u8; 256] = [0u8; 256];
static mut MOD1_KEY_MAP: [u8; 256] = [0u8; 256];
static mut MOD2_KEY_MAP: [u8; 256] = [0u8; 256];
static COMBO_KEYS: [(Pos, Pos, Kc); 3] = [
    (0x95, 0x96, KC::KBD_BACKSPACE),
    (0xa2, 0xa3, KC::KBD_ENTER),
    (0x24, 0x25, KC::KBD_ESCAPE),
];

fn expect_combo_key(on_key: u8) -> Option<(u8, u8)> {
    for (k1, k2, kc) in COMBO_KEYS.iter() {
        if on_key == *k1 {
            return Some((*k2, *kc));
        }
        if on_key == *k2 {
            return Some((*k1, *kc));
        }
    }
    return None;
}

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
        // SIMPLE_KEY_MAP[0x42] = Mod1
        // SIMPLE_KEY_MAP[0x43] = Alt
        // SIMPLE_KEY_MAP[0x44] = UI
        SIMPLE_KEY_MAP[0x45] = KC::KBD_SPACEBAR;
        // SIMPLE_KEY_MAP[0x46] = Mod2

        SIMPLE_KEY_MAP[0x91] = KC::KBD_Y;
        SIMPLE_KEY_MAP[0x92] = KC::KBD_U;
        SIMPLE_KEY_MAP[0x93] = KC::KBD_I;
        SIMPLE_KEY_MAP[0x94] = KC::KBD_O;
        SIMPLE_KEY_MAP[0x95] = KC::KBD_P;
        SIMPLE_KEY_MAP[0x96] = KC::KBD_KEYPAD_MINUS;
        SIMPLE_KEY_MAP[0xa1] = KC::KBD_H;
        SIMPLE_KEY_MAP[0xa2] = KC::KBD_J;
        SIMPLE_KEY_MAP[0xa3] = KC::KBD_K;
        SIMPLE_KEY_MAP[0xa4] = KC::KBD_L;
        SIMPLE_KEY_MAP[0xa5] = KC::KBD_COLON;
        SIMPLE_KEY_MAP[0xa6] = KC::KBD_CLOSE_BRACKET;
        SIMPLE_KEY_MAP[0xb1] = KC::KBD_N;
        SIMPLE_KEY_MAP[0xb2] = KC::KBD_M;
        SIMPLE_KEY_MAP[0xb3] = KC::KBD_DOT;
        SIMPLE_KEY_MAP[0xb4] = KC::KBD_COMMA;
        SIMPLE_KEY_MAP[0xb5] = KC::KBD_SLASH;
        SIMPLE_KEY_MAP[0xb6] = KC::KBD_BACKSLASH;
        // SIMPLE_KEY_MAP[0xc1] = Shift
        SIMPLE_KEY_MAP[0xc2] = KC::KBD_UNDERSCORE;
        // SIMPLE_KEY_MAP[0xc3] = No Key
        // SIMPLE_KEY_MAP[0xc4] = No Key
        SIMPLE_KEY_MAP[0xc5] = KC::KBD_KEYPAD_AT;
        SIMPLE_KEY_MAP[0xc6] = KC::KBD_QUOTE;

        MOD1_KEY_MAP[0x11] = KC::KBD_Y;
        MOD1_KEY_MAP[0x12] = KC::KBD_U;
        MOD1_KEY_MAP[0x13] = KC::KBD_I;
        MOD1_KEY_MAP[0x14] = KC::KBD_O;
        MOD1_KEY_MAP[0x15] = KC::KBD_P;
        MOD1_KEY_MAP[0x16] = KC::KBD_KEYPAD_MINUS;
        MOD1_KEY_MAP[0x21] = KC::KBD_H;
        MOD1_KEY_MAP[0x22] = KC::KBD_J;
        MOD1_KEY_MAP[0x23] = KC::KBD_K;
        MOD1_KEY_MAP[0x24] = KC::KBD_L;
        MOD1_KEY_MAP[0x25] = KC::KBD_COLON;
        MOD1_KEY_MAP[0x26] = KC::KBD_CLOSE_BRACKET;
        MOD1_KEY_MAP[0x31] = KC::KBD_N;
        MOD1_KEY_MAP[0x32] = KC::KBD_M;
        MOD1_KEY_MAP[0x33] = KC::KBD_DOT;
        MOD1_KEY_MAP[0x34] = KC::KBD_COMMA;
        MOD1_KEY_MAP[0x35] = KC::KBD_SLASH;
        MOD1_KEY_MAP[0x36] = KC::KBD_BACKSLASH;
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

#[cfg(test)]
mod tests {

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_feature_state_pressed() {
        let mut state = FeatureState::new();
        assert_eq!(state.pressed, [0u8; REPORT_SLOTS]);
        state.press(KC::KBD_A);
        assert_eq!(state.pressed, [KC::KBD_A, 0, 0, 0, 0, 0]);
        state.release(KC::KBD_A);
        assert_eq!(state.pressed, [0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_feature_state_pressed_multi_key() {
        let mut state = FeatureState::new();
        assert_eq!(state.pressed, [0u8; REPORT_SLOTS]);
        state.press(KC::KBD_A);
        assert_eq!(state.pressed, [KC::KBD_A, 0, 0, 0, 0, 0]);
        state.press(KC::KBD_A); // no change
        assert_eq!(state.pressed, [KC::KBD_A, 0, 0, 0, 0, 0]);
        state.press(KC::KBD_B);
        assert_eq!(state.pressed, [KC::KBD_A, KC::KBD_B, 0, 0, 0, 0]);
        state.release(KC::KBD_A);
        assert_eq!(state.pressed, [KC::KBD_B, 0, 0, 0, 0, 0]);
        state.release(KC::KBD_A); // no change
        assert_eq!(state.pressed, [KC::KBD_B, 0, 0, 0, 0, 0]);
        state.release(KC::KBD_B);
        assert_eq!(state.pressed, [0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_make_key_report() {
        let mut state = FeatureState::new();
        state.mod_key_code = KC::KBD_MODIFIER_LEFT_SHIFT;
        state.press(KC::KBD_A);
        assert_eq!(state.pressed, [KC::KBD_A, 0, 0, 0, 0, 0]);
        let key = state.make_key_report();
        assert_eq!(
            key,
            [KC::KBD_MODIFIER_LEFT_SHIFT, 0, KC::KBD_A, 0, 0, 0, 0, 0]
        );
    }
}
