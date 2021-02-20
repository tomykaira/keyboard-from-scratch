use crate::hid_keycodes as KC;
use crate::keymap::Command::{KeyPress, Nop, PressModifier};
use KC::Kc;

/// See `matrix` for encoding rule.
pub type Pos = u8;

#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub enum Command {
    Nop,
    KeyPress { kc: Kc },
    PressModifier { mk: ModifierKey },
    ModifiedKey { mk: &'static [ModifierKey], kc: Kc },
}

const fn k(kc: Kc) -> Command {
    KeyPress { kc }
}

const fn nop() -> Command {
    Nop
}

const fn m(mk: ModifierKey) -> Command {
    PressModifier { mk }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub enum ModifierKey {
    CTRL1,
    SHIFT1,
    MOD1,
    ALT1,
    UI1,
    MOD2,
    MOD3,
}

pub static SIMPLE_KEY_MAP: [Command; 48] = [
    // Left
    // R1
    k(KC::KBD_TAB),
    k(KC::KBD_Q),
    k(KC::KBD_W),
    k(KC::KBD_E),
    k(KC::KBD_R),
    k(KC::KBD_T),
    // R2
    m(ModifierKey::CTRL1),
    k(KC::KBD_A),
    k(KC::KBD_S),
    k(KC::KBD_D),
    k(KC::KBD_F),
    k(KC::KBD_G),
    // R3
    m(ModifierKey::SHIFT1),
    k(KC::KBD_Z),
    k(KC::KBD_X),
    k(KC::KBD_C),
    k(KC::KBD_V),
    k(KC::KBD_B),
    // R4
    k(KC::KBD_TILDE),
    m(ModifierKey::MOD1),
    m(ModifierKey::ALT1),
    m(ModifierKey::UI1),
    k(KC::KBD_SPACEBAR),
    m(ModifierKey::MOD2),
    // Right
    // R1
    k(KC::KBD_Y),
    k(KC::KBD_U),
    k(KC::KBD_I),
    k(KC::KBD_O),
    k(KC::KBD_P),
    k(KC::KBD_JP_HYPHEN),
    // R2
    k(KC::KBD_H),
    k(KC::KBD_J),
    k(KC::KBD_K),
    k(KC::KBD_L),
    k(KC::KBD_JP_SEMICOLON),
    k(KC::KBD_BACKSPACE),
    // R3
    k(KC::KBD_N),
    k(KC::KBD_M),
    k(KC::KBD_COMMA),
    k(KC::KBD_DOT),
    k(KC::KBD_SLASH),
    k(KC::KBD_JP_BACKSLASH),
    // R4
    m(ModifierKey::SHIFT1),
    k(KC::KBD_JP_UNDERSCORE),
    nop(),
    nop(),
    k(KC::KBD_JP_AT),
    k(KC::KBD_JP_COLON),
];

pub static MOD1_KEY_MAP: [Command; 48] = [
    // Left
    // R1
    k(KC::KBD_Y),
    k(KC::KBD_U),
    k(KC::KBD_I),
    k(KC::KBD_O),
    k(KC::KBD_P),
    k(KC::KBD_JP_HYPHEN),
    // R2
    m(ModifierKey::CTRL1),
    k(KC::KBD_J),
    k(KC::KBD_K),
    k(KC::KBD_L),
    k(KC::KBD_JP_SEMICOLON),
    k(KC::KBD_BACKSPACE),
    // R3
    m(ModifierKey::SHIFT1),
    k(KC::KBD_M),
    k(KC::KBD_COMMA),
    k(KC::KBD_DOT),
    k(KC::KBD_SLASH),
    k(KC::KBD_JP_BACKSLASH),
    // R4
    nop(),
    m(ModifierKey::MOD1),
    m(ModifierKey::ALT1),
    m(ModifierKey::UI1),
    k(KC::KBD_SPACEBAR),
    m(ModifierKey::MOD2),
    // Right
    // R1
    k(KC::KBD_TAB),
    k(KC::KBD_Q),
    k(KC::KBD_W),
    k(KC::KBD_E),
    k(KC::KBD_R),
    k(KC::KBD_T),
    // R2
    nop(),
    k(KC::KBD_A),
    k(KC::KBD_S),
    k(KC::KBD_D),
    k(KC::KBD_F),
    k(KC::KBD_G),
    // R3
    nop(),
    k(KC::KBD_Z),
    k(KC::KBD_X),
    k(KC::KBD_C),
    k(KC::KBD_V),
    k(KC::KBD_B),
    // R4
    m(ModifierKey::SHIFT1),
    nop(),
    nop(),
    nop(),
    nop(),
    nop(),
];

// JP keyboard.
static EXCLAIM: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_1,
};
static DOUBLE_QUOTE: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_2,
};
static NUMBER: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_3,
};
static DOLLAR: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_4,
};
static PERCENT: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_5,
};
static AMPERSAND: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_6,
};
static SINGLE_QUOTE: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_7,
};
static OPEN_PAREN: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_8,
};
static CLOSE_PAREN: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_9,
};
static EQUAL: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_JP_HYPHEN,
};
pub static ASTERISK: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_JP_COLON,
};
static CMD_LBRACE: Command = Command::ModifiedKey {
    mk: &[ModifierKey::UI1, ModifierKey::SHIFT1],
    kc: KC::KBD_JP_OPEN_BRACKET,
};
static CMD_RBRACE: Command = Command::ModifiedKey {
    mk: &[ModifierKey::UI1, ModifierKey::SHIFT1],
    kc: KC::KBD_JP_CLOSE_BRACKET,
};
static LBRACE: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_JP_OPEN_BRACKET,
};
static RBRACE: Command = Command::ModifiedKey {
    mk: &[ModifierKey::SHIFT1],
    kc: KC::KBD_JP_CLOSE_BRACKET,
};

pub static MOD2_KEY_MAP: [Command; 48] = [
    // Left
    // R1
    nop(),
    EXCLAIM,
    DOUBLE_QUOTE,
    NUMBER,
    DOLLAR,
    PERCENT,
    // R2
    m(ModifierKey::CTRL1),
    nop(),
    nop(),
    nop(),
    nop(),
    DOLLAR,
    // R3
    m(ModifierKey::SHIFT1),
    nop(),
    nop(),
    nop(),
    nop(),
    nop(),
    // R4
    nop(),
    m(ModifierKey::MOD1),
    m(ModifierKey::ALT1),
    m(ModifierKey::UI1),
    nop(),
    m(ModifierKey::MOD2),
    // Right
    // R1
    AMPERSAND,
    SINGLE_QUOTE,
    OPEN_PAREN,
    CLOSE_PAREN,
    nop(),
    EQUAL,
    // R2
    k(KC::KBD_LEFT),
    k(KC::KBD_DOWN),
    k(KC::KBD_UP),
    k(KC::KBD_RIGHT),
    nop(),
    nop(),
    // R3
    nop(),
    nop(),
    nop(),
    nop(),
    nop(),
    nop(),
    // R4
    m(ModifierKey::SHIFT1),
    nop(),
    nop(),
    nop(),
    nop(),
    nop(),
];

pub static MOD3_KEY_MAP: [Command; 48] = [
    // Left
    // R1
    nop(),
    k(KC::KBD_F9),
    k(KC::KBD_F10),
    k(KC::KBD_F11),
    k(KC::KBD_F12),
    nop(),
    // R2
    m(ModifierKey::CTRL1),
    k(KC::KBD_F5),
    k(KC::KBD_F6),
    k(KC::KBD_F7),
    k(KC::KBD_F8),
    nop(),
    // R3
    m(ModifierKey::SHIFT1),
    k(KC::KBD_F1),
    k(KC::KBD_F2),
    k(KC::KBD_F3),
    k(KC::KBD_F4),
    nop(),
    // R4
    nop(),
    m(ModifierKey::MOD1),
    m(ModifierKey::ALT1),
    m(ModifierKey::UI1),
    nop(),
    m(ModifierKey::MOD2),
    // Right
    // R1
    nop(),
    k(KC::KBD_7),
    k(KC::KBD_8),
    k(KC::KBD_9),
    ASTERISK,
    nop(),
    // R2
    nop(),
    k(KC::KBD_4),
    k(KC::KBD_5),
    k(KC::KBD_6),
    k(KC::KBD_KEYPAD_PLUS),
    nop(),
    // R3
    nop(),
    k(KC::KBD_1),
    k(KC::KBD_2),
    k(KC::KBD_3),
    k(KC::KBD_0),
    nop(),
    // R4
    m(ModifierKey::SHIFT1),
    nop(),
    nop(),
    nop(),
    nop(),
    nop(),
];

pub static COMBO_KEYS: [(Pos, Pos, Command); 9] = [
    (0xa2, 0xa3, k(KC::KBD_ENTER)),
    (0x24, 0x25, k(KC::KBD_ESCAPE)),
    (0x44, 0x45, m(ModifierKey::MOD3)),
    (0xa4, 0xa5, k(KC::KBD_JP_OPEN_BRACKET)),
    (0xa5, 0xa6, k(KC::KBD_JP_CLOSE_BRACKET)),
    (0xb2, 0xb3, LBRACE),
    (0xb3, 0xb4, RBRACE),
    (0x93, 0x95, CMD_LBRACE),
    (0xa3, 0xa5, CMD_RBRACE),
];
