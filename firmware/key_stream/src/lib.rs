#![no_std]
#![deny(warnings)]

#[cfg(test)]
#[macro_use]
extern crate std;

mod hid_keycodes;
mod keymap;
pub mod ring_buffer;

use crate::hid_keycodes as KC;
use crate::keymap::*;
use crate::ring_buffer::RingBuffer;

const REPORT_SLOTS: usize = 6;
const N_COL: u8 = 6;
#[allow(dead_code)]
const N_ROW: u8 = 4;

impl ModifierKey {
    pub fn code(&self) -> u8 {
        match *self {
            ModifierKey::CTRL1 => KC::KBD_MODIFIER_LEFT_CTRL,
            ModifierKey::SHIFT1 => KC::KBD_MODIFIER_LEFT_SHIFT,
            ModifierKey::MOD1 => 0,
            ModifierKey::ALT1 => KC::KBD_MODIFIER_LEFT_ALT,
            ModifierKey::UI1 => KC::KBD_MODIFIER_LEFT_UI,
            ModifierKey::MOD2 => 0,
            ModifierKey::MOD3 => 0,
        }
    }
}

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
    mods: [bool; 3],
    commands: [Command; REPORT_SLOTS],
}

impl FeatureState {
    fn new() -> FeatureState {
        FeatureState {
            mods: [false; 3],
            commands: [Command::Nop; REPORT_SLOTS],
        }
    }

    /// Process newly activated command.
    /// Return true if HID report will change.
    fn press(&mut self, command: &Command) -> bool {
        match command {
            Command::Nop => false,
            Command::PressModifier {
                mk: ModifierKey::MOD1,
            } => {
                self.mods[0] = true;
                false
            }
            Command::PressModifier {
                mk: ModifierKey::MOD2,
            } => {
                self.mods[1] = true;
                false
            }
            Command::PressModifier {
                mk: ModifierKey::MOD3,
            } => {
                self.mods[2] = true;
                false
            }
            other => {
                self.push_key_command(other);
                true
            }
        }
    }

    fn push_key_command(&mut self, command: &Command) {
        for i in 0..REPORT_SLOTS {
            if self.commands[i] == *command {
                return;
            }
            if !self.commands[i].is_defined() {
                self.commands[i] = *command;
                return;
            }
        }
        // no more slots.
    }

    /// Process newly deactivated command.
    /// Return true if HID report will change.
    fn release(&mut self, command: &Command) -> bool {
        match command {
            Command::Nop => false,
            Command::PressModifier {
                mk: ModifierKey::MOD1,
            } => {
                self.mods[0] = false;
                false
            }
            Command::PressModifier {
                mk: ModifierKey::MOD2,
            } => {
                self.mods[1] = false;
                false
            }
            Command::PressModifier {
                mk: ModifierKey::MOD3,
            } => {
                self.mods[2] = false;
                false
            }
            other => {
                self.pop_key_command(other);
                true
            }
        }
    }

    fn pop_key_command(&mut self, command: &Command) {
        for i in 0..REPORT_SLOTS {
            if self.commands[i] == *command {
                for j in i..REPORT_SLOTS - 1 {
                    self.commands[j] = self.commands[j + 1];
                }
                self.commands[REPORT_SLOTS - 1] = Command::Nop;
                return;
            }
            if !self.commands[i].is_defined() {
                return;
            }
        }
    }

    fn make_key_report(&self) -> [u8; 8] {
        let mut key = [0u8; 8];
        let mut ptr = 2;
        for c in self.commands.iter() {
            match c {
                Command::Nop => {}
                Command::KeyPress { kc } => {
                    key[ptr] = *kc;
                    ptr += 1;
                }
                Command::PressModifier { mk } => {
                    key[0] |= mk.code();
                }
                Command::ModifiedKey { mk, kc } => {
                    key[ptr] = *kc;
                    ptr += 1;
                    key[0] |= mk.code();
                }
            }
        }
        return key;
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Action {
    DOWN,
    UP,
}

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
            Action::DOWN => {
                if let Some(command) = self.process_combo_keys(ev.pos) {
                    if self.state.press(command) {
                        emit(self.state.make_key_report());
                    }
                    self.consume_event(); // skip one more event.
                } else {
                    let idx = pos_to_map_index(ev.pos);
                    let map = if self.state.mods[0] {
                        &MOD1_KEY_MAP
                    } else if self.state.mods[1] {
                        &MOD2_KEY_MAP
                    } else if self.state.mods[2] {
                        &MOD3_KEY_MAP
                    } else {
                        &SIMPLE_KEY_MAP
                    };
                    let k = &map[idx];
                    if self.state.press(k) {
                        emit(self.state.make_key_report());
                    }
                }
            }
            Action::UP => {
                self.release_related_keys(ev.pos);
            }
        }
        self.consume_event();
    }

    fn release_related_keys(&mut self, pos: Pos) {
        let idx = pos_to_map_index(pos);
        let k = &MOD1_KEY_MAP[idx];
        self.state.release(k);
        let k = &MOD2_KEY_MAP[idx];
        self.state.release(k);
        let k = &MOD3_KEY_MAP[idx];
        self.state.release(k);
        let k = &SIMPLE_KEY_MAP[idx];
        self.state.release(k);
        for (k1, k2, kc) in COMBO_KEYS.iter() {
            if pos == *k1 || pos == *k2 {
                self.state.release(kc);
            }
        }
    }

    // TODO: wait the next key crossing transform period.
    fn process_combo_keys(&self, pos: Pos) -> Option<&'static Command> {
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

static VALID_KEY_LIST: [Pos; 48] = [
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x31, 0x32, 0x33, 0x34,
    0x35, 0x36, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0xa1, 0xa2,
    0xa3, 0xa4, 0xa5, 0xa6, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6,
];

fn expect_combo_key(on_key: Pos) -> Option<(Pos, &'static Command)> {
    for (k1, k2, kc) in COMBO_KEYS.iter() {
        if on_key == *k1 {
            return Some((*k2, kc));
        }
        if on_key == *k2 {
            return Some((*k1, kc));
        }
    }
    return None;
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

fn pos_to_map_index(pos: Pos) -> usize {
    let row = pos >> 4;
    let col = pos & 0x0f;
    let i = if row >= 9 { row - 9 + 4 } else { row - 1 };
    let j = col - 1;
    (i * N_COL + j) as usize
}

impl Command {
    fn is_defined(&self) -> bool {
        match self {
            Command::Nop => false,
            Command::KeyPress { .. } => true,
            Command::PressModifier { .. } => true,
            Command::ModifiedKey { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_to_map_index() {
        assert_eq!(pos_to_map_index(0x11), 0);
        assert_eq!(pos_to_map_index(0x16), 5);
        assert_eq!(pos_to_map_index(0x46), 23);
        assert_eq!(pos_to_map_index(0x91), 24);
        assert_eq!(pos_to_map_index(0xa1), 30);
        assert_eq!(pos_to_map_index(0xc6), 47);
    }

    #[test]
    fn test_feature_state_pressed() {
        let a = Command::KeyPress { kc: KC::KBD_A };

        let mut state = FeatureState::new();
        assert_eq!(state.make_key_report(), [0, 0, 0, 0, 0, 0, 0, 0]);
        state.press(&a);
        assert_eq!(state.make_key_report(), [0, 0, KC::KBD_A, 0, 0, 0, 0, 0]);
        state.release(&a);
        assert_eq!(state.make_key_report(), [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_feature_state_pressed_multi_key() {
        let a = Command::KeyPress { kc: KC::KBD_A };
        let b = Command::KeyPress { kc: KC::KBD_B };

        let mut state = FeatureState::new();
        assert_eq!(state.make_key_report(), [0, 0, 0, 0, 0, 0, 0, 0]);

        state.press(&a);
        assert_eq!(state.make_key_report(), [0, 0, KC::KBD_A, 0, 0, 0, 0, 0]);

        state.press(&a); // no change
        assert_eq!(state.make_key_report(), [0, 0, KC::KBD_A, 0, 0, 0, 0, 0]);

        state.press(&b);
        assert_eq!(
            state.make_key_report(),
            [0, 0, KC::KBD_A, KC::KBD_B, 0, 0, 0, 0]
        );

        state.release(&a);
        assert_eq!(state.make_key_report(), [0, 0, KC::KBD_B, 0, 0, 0, 0, 0]);

        state.release(&a); // no change
        assert_eq!(state.make_key_report(), [0, 0, KC::KBD_B, 0, 0, 0, 0, 0]);

        state.release(&b);
        assert_eq!(state.make_key_report(), [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_feature_state_modified_key() {
        let shift = Command::PressModifier {
            mk: ModifierKey::SHIFT1,
        };
        let a = Command::KeyPress { kc: KC::KBD_A };

        let mut state = FeatureState::new();
        state.press(&shift);
        state.press(&a);
        assert_eq!(
            state.make_key_report(),
            [KC::KBD_MODIFIER_LEFT_SHIFT, 0, KC::KBD_A, 0, 0, 0, 0, 0]
        );

        state.press(&ASTERISK);
        assert_eq!(
            state.make_key_report(),
            [
                KC::KBD_MODIFIER_LEFT_SHIFT,
                0,
                KC::KBD_A,
                KC::KBD_JP_COLON,
                0,
                0,
                0,
                0
            ]
        );

        state.release(&ASTERISK);
        assert_eq!(
            state.make_key_report(),
            [KC::KBD_MODIFIER_LEFT_SHIFT, 0, KC::KBD_A, 0, 0, 0, 0, 0]
        );

        state.release(&a);
        assert_eq!(
            state.make_key_report(),
            [KC::KBD_MODIFIER_LEFT_SHIFT, 0, 0, 0, 0, 0, 0, 0]
        );

        state.release(&shift);
        assert_eq!(state.make_key_report(), [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_feature_state_modified_key2() {
        let shift = Command::PressModifier {
            mk: ModifierKey::SHIFT1,
        };
        let a = Command::KeyPress { kc: KC::KBD_A };

        let mut state = FeatureState::new();
        state.press(&shift);
        state.press(&a);
        assert_eq!(
            state.make_key_report(),
            [KC::KBD_MODIFIER_LEFT_SHIFT, 0, KC::KBD_A, 0, 0, 0, 0, 0]
        );

        state.press(&ASTERISK);
        assert_eq!(
            state.make_key_report(),
            [
                KC::KBD_MODIFIER_LEFT_SHIFT,
                0,
                KC::KBD_A,
                KC::KBD_JP_COLON,
                0,
                0,
                0,
                0
            ]
        );

        println!("{:?}", state.commands);
        state.release(&a);
        println!("{:?}", state.commands);
        assert_eq!(
            state.make_key_report(),
            [
                KC::KBD_MODIFIER_LEFT_SHIFT,
                0,
                KC::KBD_JP_COLON,
                0,
                0,
                0,
                0,
                0
            ]
        );

        state.release(&shift);
        assert_eq!(
            state.make_key_report(),
            [
                KC::KBD_MODIFIER_LEFT_SHIFT,
                0,
                KC::KBD_JP_COLON,
                0,
                0,
                0,
                0,
                0
            ],
        );

        state.release(&ASTERISK);
        assert_eq!(state.make_key_report(), [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_feature_state_internal_mod() {
        let mod1 = Command::PressModifier {
            mk: ModifierKey::MOD1,
        };
        let mod2 = Command::PressModifier {
            mk: ModifierKey::MOD2,
        };

        let mut state = FeatureState::new();
        state.press(&mod1);
        assert_eq!(state.mods, [true, false, false]);
        state.press(&mod2);
        assert_eq!(state.mods, [true, true, false]);
        state.release(&mod1);
        assert_eq!(state.mods, [false, true, false]);
        state.release(&mod2);
        assert_eq!(state.mods, [false, false, false]);
    }
}
