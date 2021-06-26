#![cfg_attr(target_arch = "arm", no_std)]
#![deny(warnings)]

#[cfg(target_arch = "x86_64")]
extern crate std;

#[cfg(target_arch = "x86_64")]
#[allow(unused_imports)]
use std::println;

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
const COMBO_THRESHOLD_CNT: u16 = 219; // * 65536 / 72000 = 200
const COMBO_SEPARATION_CNT: u16 = 0; // * 65536 / 72000 = 500

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
    last_action_cnt: u16,
    requests_reset: bool,
}

impl FeatureState {
    fn new() -> FeatureState {
        FeatureState {
            mods: [false; 3],
            commands: [Command::Nop; REPORT_SLOTS],
            last_action_cnt: 0,
            requests_reset: false,
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

    fn make_key_report(&mut self) -> [u8; 8] {
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
                    for m in mk.iter() {
                        key[0] |= m.code();
                    }
                }
                Command::RequestReset => {
                    self.requests_reset = true;
                }
            }
        }
        return key;
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
enum Action {
    DOWN,
    UP,
}

/// Key event struct.
#[derive(Copy, Clone)]
#[cfg_attr(test, derive(Debug))]
struct Event {
    action: Action,
    pos: Pos,
    cnt: u16, // 1/65536 cnt = 1/72 us
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
        // Process releases first, then press.
        for i in &VALID_KEY_LIST {
            let on = is_on(mat, peer, *i);
            let was_on = self.on_pos[*i as usize];
            if was_on && !on {
                self.push_event(&Event {
                    action: Action::UP,
                    pos: *i,
                    cnt,
                });
                self.on_pos[*i as usize] = on;
            }
        }
        for i in &VALID_KEY_LIST {
            let on = is_on(mat, peer, *i);
            let was_on = self.on_pos[*i as usize];
            if !was_on && on {
                self.push_event(&Event {
                    action: Action::DOWN,
                    pos: *i,
                    cnt,
                });
                self.on_pos[*i as usize] = on;
            }
        }
    }

    fn push_event(&mut self, evt: &Event) {
        self.events.push(evt)
    }

    pub fn requests_reset(&self) -> bool {
        self.state.requests_reset
    }

    /// Return: `[modifier, key]`
    pub fn read<F>(&mut self, clk: u32, mut emit: F)
    where
        F: FnMut([u8; 8]) -> (),
    {
        let cnt = (clk >> 16) as u16;
        let mut executed = false;
        while let Some(ev) = self.peek_event(0) {
            let (e, consumed) = self.proc_event(cnt, &ev, &mut emit);
            executed = executed || e || !consumed;
            if !consumed {
                break;
            }
        }
        if !executed {
            emit(self.state.make_key_report());
        }
    }

    /// return true if emit is called.
    fn proc_event<F>(&mut self, cnt: u16, ev: &Event, mut emit: F) -> (bool, bool)
    where
        F: FnMut([u8; 8]) -> (),
    {
        if ev.pos == 0 {
            // skip pos = 0, empty event.
            self.consume_event();
            return (false, true);
        }

        match ev.action {
            Action::DOWN => {
                match self.process_combo_keys(cnt, ev) {
                    ComboKeyResult::ProcessCombo { command } => {
                        if self.state.press(command) {
                            emit(self.state.make_key_report());
                        }
                        self.consume_event(); // consume two keys
                        self.consume_event();
                        (true, true)
                    }
                    ComboKeyResult::Wait => (false, false),
                    ComboKeyResult::NotCombo => {
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
                            self.state.last_action_cnt = cnt;
                            emit(self.state.make_key_report());
                        }
                        self.consume_event();
                        (true, true)
                    }
                }
            }
            Action::UP => {
                self.release_related_keys(ev.pos);
                self.consume_event();
                (false, true)
            }
        }
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

    fn process_combo_keys(&self, now_cnt: u16, event: &Event) -> ComboKeyResult {
        let pos = event.pos;
        // Ignore key combo in sequence of keys - such as typing words.
        if self.state.last_action_cnt + COMBO_SEPARATION_CNT > now_cnt {
            ComboKeyResult::NotCombo
        } else if event.cnt + COMBO_THRESHOLD_CNT < now_cnt {
            ComboKeyResult::NotCombo
        } else if expect_combo_key(pos) {
            if let Some(next) = self.peek_event(1) {
                if let Some(command) = find_combo(pos, next.pos) {
                    ComboKeyResult::ProcessCombo { command }
                } else {
                    ComboKeyResult::NotCombo
                }
            } else {
                ComboKeyResult::Wait
            }
        } else {
            ComboKeyResult::NotCombo
        }
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

enum ComboKeyResult {
    ProcessCombo { command: &'static Command },
    Wait,
    NotCombo,
}

static VALID_KEY_LIST: [Pos; 48] = [
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x31, 0x32, 0x33, 0x34,
    0x35, 0x36, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0xa1, 0xa2,
    0xa3, 0xa4, 0xa5, 0xa6, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6,
];

fn expect_combo_key(on_key: Pos) -> bool {
    for (k1, k2, _) in COMBO_KEYS.iter() {
        if on_key == *k1 {
            return true;
        }
        if on_key == *k2 {
            return true;
        }
    }
    return false;
}

fn find_combo(on_k1: Pos, on_k2: Pos) -> Option<&'static Command> {
    for (k1, k2, kc) in COMBO_KEYS.iter() {
        if on_k1 == *k1 && on_k2 == *k2 || on_k1 == *k2 && on_k2 == *k1 {
            return Some(kc);
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
            Command::RequestReset { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;
    const COMBO_THRESHOLD_MS: u32 = 200;
    const COMBO_SEPARATION_MS: u32 = 500;

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

        state.release(&a);
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

    // Convert millisecond to clock with arbitrary offset.
    fn ms(ms: u32) -> u32 {
        (1204 + ms) * 72_000
    }

    struct MockEmit {
        history: Vec<[u8; 8]>,
    }

    fn mock_emit() -> MockEmit {
        MockEmit { history: vec![] }
    }

    impl MockEmit {
        fn emit(&mut self, v: [u8; 8]) {
            self.history.push(v)
        }
        fn verify(&self, expected: Vec<[u8; 8]>) {
            assert_eq!(self.history, expected);
        }
    }

    #[test]
    fn test_key_stream_simple_key_in() {
        let mut stream = KeyStream::new();
        let mut e = mock_emit();
        stream.push(&[0x22, 0, 0, 0, 0, 0, 0, 0], &[0u8; 8], ms(100));
        stream.read(ms(101), |x| e.emit(x));
        e.verify(vec![[0, 0, KC::KBD_A, 0, 0, 0, 0, 0]]);
    }

    #[test]
    fn test_key_stream_combo_key_flash_by_time() {
        let mut stream = KeyStream::new();
        let mut e = mock_emit();
        let semicolon = [0, 0, KC::KBD_JP_SEMICOLON, 0, 0, 0, 0, 0];

        stream.push(&[0u8; 8], &[0xa5, 0, 0, 0, 0, 0, 0, 0], ms(0));
        stream.read(ms(1), |x| e.emit(x));
        e.verify(vec![]);

        // still wait
        stream.push(
            &[0u8; 8],
            &[0xa5, 0, 0, 0, 0, 0, 0, 0],
            ms(COMBO_THRESHOLD_MS - 1),
        );
        stream.read(ms(COMBO_THRESHOLD_MS - 1), |x| e.emit(x));
        e.verify(vec![]);

        stream.push(
            &[0u8; 8],
            &[0xa5, 0, 0, 0, 0, 0, 0, 0],
            ms(COMBO_THRESHOLD_MS + 1),
        );

        stream.read(ms(COMBO_THRESHOLD_MS + 1), |x| e.emit(x));
        e.verify(vec![semicolon]);

        // key repeat
        stream.push(
            &[0u8; 8],
            &[0xa5, 0, 0, 0, 0, 0, 0, 0],
            ms(COMBO_THRESHOLD_MS + 2),
        );
        stream.read(ms(COMBO_THRESHOLD_MS + 2), |x| e.emit(x));
        e.verify(vec![semicolon, semicolon]);
    }

    #[test]
    fn test_key_stream_combo_no_pause() {
        let mut stream = KeyStream::new();
        let mut e = mock_emit();
        let a = [0, 0, KC::KBD_A, 0, 0, 0, 0, 0];
        let semi = [0, 0, KC::KBD_JP_SEMICOLON, 0, 0, 0, 0, 0];
        let semi_bksp = [0, 0, KC::KBD_JP_SEMICOLON, KC::KBD_BACKSPACE, 0, 0, 0, 0];

        stream.push(&[0u8; 8], &[0x22, 0, 0, 0, 0, 0, 0, 0], ms(0));
        stream.read(ms(1), |x| e.emit(x));
        e.verify(vec![a]);

        // down key combo
        stream.push(&[0u8; 8], &[0xa5, 0xa6, 0, 0, 0, 0, 0, 0], ms(2));
        stream.read(ms(3), |x| e.emit(x));
        e.verify(vec![a, semi, semi_bksp]);
    }

    #[test]
    fn test_key_stream_combo_after_pause() {
        let mut stream = KeyStream::new();
        let mut e = mock_emit();
        let a = [0, 0, KC::KBD_A, 0, 0, 0, 0, 0];
        let bracket = [0, 0, KC::KBD_JP_CLOSE_BRACKET, 0, 0, 0, 0, 0];

        stream.push(&[0u8; 8], &[0x22, 0, 0, 0, 0, 0, 0, 0], ms(0));
        stream.read(ms(1), |x| e.emit(x));
        e.verify(vec![a]);

        // down key combo
        stream.push(
            &[0u8; 8],
            &[0xa5, 0xa6, 0, 0, 0, 0, 0, 0],
            ms(1 + COMBO_SEPARATION_MS),
        );
        stream.read(ms(1 + COMBO_SEPARATION_MS + COMBO_THRESHOLD_MS), |x| {
            e.emit(x)
        });
        e.verify(vec![a, bracket]);
    }

    #[test]
    fn test_key_stream_combo_key_flash_by_release() {
        let mut stream = KeyStream::new();
        let mut e = mock_emit();
        let semicolon = [0, 0, KC::KBD_JP_SEMICOLON, 0, 0, 0, 0, 0];

        stream.push(&[0u8; 8], &[0xa5, 0, 0, 0, 0, 0, 0, 0], ms(0));
        stream.read(ms(1), |x| e.emit(x));
        e.verify(vec![]);

        // flash rightly because released
        stream.push(&[0u8; 8], &[0u8; 8], ms(1));
        stream.read(ms(2), |x| e.emit(x));
        e.verify(vec![semicolon]);
    }

    #[test]
    fn test_key_stream_combo_key_flash_by_other_key() {
        let mut stream = KeyStream::new();
        let mut e = mock_emit();
        let semicolon = [0, 0, KC::KBD_JP_SEMICOLON, 0, 0, 0, 0, 0];
        let a = [0, 0, KC::KBD_A, 0, 0, 0, 0, 0];

        stream.push(&[0u8; 8], &[0xa5, 0, 0, 0, 0, 0, 0, 0], ms(0));
        stream.read(ms(1), |x| e.emit(x));
        e.verify(vec![]);

        // flash rightly because non combo key is pressed
        stream.push(&[0x22, 0, 0, 0, 0, 0, 0, 0], &[0u8; 8], ms(1));
        stream.read(ms(2), |x| e.emit(x));
        e.verify(vec![semicolon, a]);
    }

    #[test]
    fn test_key_stream_combo_in_one_scan() {
        let mut stream = KeyStream::new();
        let mut e = mock_emit();
        let bracket = [0, 0, KC::KBD_JP_CLOSE_BRACKET, 0, 0, 0, 0, 0];
        let zero = [0, 0, 0, 0, 0, 0, 0, 0];

        stream.push(&[0u8; 8], &[0xa5, 0, 0, 0, 0, 0, 0, 0], ms(0));
        stream.push(&[0u8; 8], &[0xa5, 0xa6, 0, 0, 0, 0, 0, 0], ms(1));
        stream.push(&[0u8; 8], &[0xa6, 0, 0, 0, 0, 0, 0, 0], ms(2));
        stream.push(&[0u8; 8], &[0, 0, 0, 0, 0, 0, 0, 0], ms(3));
        stream.read(ms(4), |x| e.emit(x));
        e.verify(vec![bracket]);

        stream.push(&[0u8; 8], &[0u8; 8], ms(5));
        stream.read(ms(COMBO_THRESHOLD_MS + 10), |x| e.emit(x));
        e.verify(vec![bracket, zero]); // no new key
    }

    #[test]
    fn test_key_stream_combo_in_two_scans() {
        let mut stream = KeyStream::new();
        let mut e = mock_emit();
        let bracket = [0, 0, KC::KBD_JP_CLOSE_BRACKET, 0, 0, 0, 0, 0];
        let zero = [0, 0, 0, 0, 0, 0, 0, 0];

        stream.push(&[0u8; 8], &[0xa5, 0, 0, 0, 0, 0, 0, 0], ms(0));
        stream.read(ms(0), |x| e.emit(x));
        stream.push(&[0u8; 8], &[0xa5, 0xa6, 0, 0, 0, 0, 0, 0], ms(1));
        stream.read(ms(1), |x| e.emit(x));
        e.verify(vec![bracket]);
        stream.push(&[0u8; 8], &[0xa6, 0, 0, 0, 0, 0, 0, 0], ms(2));
        stream.read(ms(2), |x| e.emit(x));
        e.verify(vec![bracket, zero]);
        stream.push(&[0u8; 8], &[0, 0, 0, 0, 0, 0, 0, 0], ms(3));
        stream.read(ms(3), |x| e.emit(x));
        e.verify(vec![bracket, zero, zero]);
        stream.read(ms(4), |x| e.emit(x));
        e.verify(vec![bracket, zero, zero, zero]);
    }
}
