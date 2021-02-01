/// See `hid_keycodes` for mapping.
pub type Kc = u8;

#[allow(dead_code)]
pub static KBD_NONE: Kc = 0;
#[allow(dead_code)]
pub static KBD_A: Kc = 4;
#[allow(dead_code)]
pub static KBD_B: Kc = 5;
#[allow(dead_code)]
pub static KBD_C: Kc = 6;
#[allow(dead_code)]
pub static KBD_D: Kc = 7;
#[allow(dead_code)]
pub static KBD_E: Kc = 8;
#[allow(dead_code)]
pub static KBD_F: Kc = 9;
#[allow(dead_code)]
pub static KBD_G: Kc = 10;
#[allow(dead_code)]
pub static KBD_H: Kc = 11;
#[allow(dead_code)]
pub static KBD_I: Kc = 12;
#[allow(dead_code)]
pub static KBD_J: Kc = 13;
#[allow(dead_code)]
pub static KBD_K: Kc = 14;
#[allow(dead_code)]
pub static KBD_L: Kc = 15;
#[allow(dead_code)]
pub static KBD_M: Kc = 16;
#[allow(dead_code)]
pub static KBD_N: Kc = 17;
#[allow(dead_code)]
pub static KBD_O: Kc = 18;
#[allow(dead_code)]
pub static KBD_P: Kc = 19;
#[allow(dead_code)]
pub static KBD_Q: Kc = 20;
#[allow(dead_code)]
pub static KBD_R: Kc = 21;
#[allow(dead_code)]
pub static KBD_S: Kc = 22;
#[allow(dead_code)]
pub static KBD_T: Kc = 23;
#[allow(dead_code)]
pub static KBD_U: Kc = 24;
#[allow(dead_code)]
pub static KBD_V: Kc = 25;
#[allow(dead_code)]
pub static KBD_W: Kc = 26;
#[allow(dead_code)]
pub static KBD_X: Kc = 27;
#[allow(dead_code)]
pub static KBD_Y: Kc = 28;
#[allow(dead_code)]
pub static KBD_Z: Kc = 29;
#[allow(dead_code)]
pub static KBD_1: Kc = 30;
#[allow(dead_code)]
pub static KBD_2: Kc = 31;
#[allow(dead_code)]
pub static KBD_3: Kc = 32;
#[allow(dead_code)]
pub static KBD_4: Kc = 33;
#[allow(dead_code)]
pub static KBD_5: Kc = 34;
#[allow(dead_code)]
pub static KBD_6: Kc = 35;
#[allow(dead_code)]
pub static KBD_7: Kc = 36;
#[allow(dead_code)]
pub static KBD_8: Kc = 37;
#[allow(dead_code)]
pub static KBD_9: Kc = 38;
#[allow(dead_code)]
pub static KBD_0: Kc = 39;
#[allow(dead_code)]
pub static KBD_ENTER: Kc = 40;
#[allow(dead_code)]
pub static KBD_ESCAPE: Kc = 41;
#[allow(dead_code)]
pub static KBD_BACKSPACE: Kc = 42;
#[allow(dead_code)]
pub static KBD_TAB: Kc = 43;
#[allow(dead_code)]
pub static KBD_SPACEBAR: Kc = 44;
#[allow(dead_code)]
pub static KBD_UNDERSCORE: Kc = 45;
#[allow(dead_code)]
pub static KBD_JP_HYPHEN: Kc = 45; // - / =
#[allow(dead_code)]
pub static KBD_PLUS: Kc = 46;
#[allow(dead_code)]
pub static KBD_JP_CARET: Kc = 46; // ^ / ~
#[allow(dead_code)]
pub static KBD_OPEN_BRACKET: Kc = 47;
#[allow(dead_code)]
pub static KBD_JP_AT: Kc = 47; // @ / `
#[allow(dead_code)]
pub static KBD_CLOSE_BRACKET: Kc = 48;
#[allow(dead_code)]
pub static KBD_JP_OPEN_BRACKET: Kc = 48; // [ / {
#[allow(dead_code)]
pub static KBD_BACKSLASH: Kc = 49;
#[allow(dead_code)]
pub static KBD_JP_CLOSE_BLACKET: Kc = 49; // ] / }
#[allow(dead_code)]
pub static KBD_ASH: Kc = 50;
#[allow(dead_code)]
pub static KBD_COLON: Kc = 51;
#[allow(dead_code)]
pub static KBD_JP_SEMICOLON: Kc = 51; // ; / +
#[allow(dead_code)]
pub static KBD_QUOTE: Kc = 52;
#[allow(dead_code)]
pub static KBD_JP_COLON: Kc = 52; // : / *
#[allow(dead_code)]
pub static KBD_TILDE: Kc = 53;
#[allow(dead_code)]
pub static KBD_COMMA: Kc = 54;
#[allow(dead_code)]
pub static KBD_DOT: Kc = 55;
#[allow(dead_code)]
pub static KBD_SLASH: Kc = 56;
#[allow(dead_code)]
pub static KBD_CAPS_LOCK: Kc = 57;
#[allow(dead_code)]
pub static KBD_F1: Kc = 58;
#[allow(dead_code)]
pub static KBD_F2: Kc = 59;
#[allow(dead_code)]
pub static KBD_F3: Kc = 60;
#[allow(dead_code)]
pub static KBD_F4: Kc = 61;
#[allow(dead_code)]
pub static KBD_F5: Kc = 62;
#[allow(dead_code)]
pub static KBD_F6: Kc = 63;
#[allow(dead_code)]
pub static KBD_F7: Kc = 64;
#[allow(dead_code)]
pub static KBD_F8: Kc = 65;
#[allow(dead_code)]
pub static KBD_F9: Kc = 66;
#[allow(dead_code)]
pub static KBD_F10: Kc = 67;
#[allow(dead_code)]
pub static KBD_F11: Kc = 68;
#[allow(dead_code)]
pub static KBD_F12: Kc = 69;
#[allow(dead_code)]
pub static KBD_PRINTSCREEN: Kc = 70;
#[allow(dead_code)]
pub static KBD_SCROLL_LOCK: Kc = 71;
#[allow(dead_code)]
pub static KBD_PAUSE: Kc = 72;
#[allow(dead_code)]
pub static KBD_INSERT: Kc = 73;
#[allow(dead_code)]
pub static KBD_HOME: Kc = 74;
#[allow(dead_code)]
pub static KBD_PAGEUP: Kc = 75;
#[allow(dead_code)]
pub static KBD_DELETE: Kc = 76;
#[allow(dead_code)]
pub static KBD_END: Kc = 77;
#[allow(dead_code)]
pub static KBD_PAGEDOWN: Kc = 78;
#[allow(dead_code)]
pub static KBD_RIGHT: Kc = 79;
#[allow(dead_code)]
pub static KBD_LEFT: Kc = 80;
#[allow(dead_code)]
pub static KBD_DOWN: Kc = 81;
#[allow(dead_code)]
pub static KBD_UP: Kc = 82;
#[allow(dead_code)]
pub static KBD_KEYPAD_NUM_LOCK: Kc = 83;
#[allow(dead_code)]
pub static KBD_KEYPAD_DIVIDE: Kc = 84;
#[allow(dead_code)]
pub static KBD_KEYPAD_AT: Kc = 85;
#[allow(dead_code)]
pub static KBD_KEYPAD_MULTIPLY: Kc = 85;
#[allow(dead_code)]
pub static KBD_KEYPAD_MINUS: Kc = 86;
#[allow(dead_code)]
pub static KBD_KEYPAD_PLUS: Kc = 87;
#[allow(dead_code)]
pub static KBD_KEYPAD_ENTER: Kc = 88;
#[allow(dead_code)]
pub static KBD_KEYPAD_1: Kc = 89;
#[allow(dead_code)]
pub static KBD_KEYPAD_2: Kc = 90;
#[allow(dead_code)]
pub static KBD_KEYPAD_3: Kc = 91;
#[allow(dead_code)]
pub static KBD_KEYPAD_4: Kc = 92;
#[allow(dead_code)]
pub static KBD_KEYPAD_5: Kc = 93;
#[allow(dead_code)]
pub static KBD_KEYPAD_6: Kc = 94;
#[allow(dead_code)]
pub static KBD_KEYPAD_7: Kc = 95;
#[allow(dead_code)]
pub static KBD_KEYPAD_8: Kc = 96;
#[allow(dead_code)]
pub static KBD_KEYPAD_9: Kc = 97;
#[allow(dead_code)]
pub static KBD_KEYPAD_0: Kc = 98;

#[allow(dead_code)]
pub static KBD_JP_BACKSLASH: Kc = 137; // \(¥) / |
#[allow(dead_code)]
pub static KBD_JP_UNDERSCORE: Kc = 135;
#[allow(dead_code)]
pub static KBD_JP_MUHENKAN: Kc = 139;
#[allow(dead_code)]
pub static KBD_JP_HANKAKU_ZENAKKU: Kc = 138;
#[allow(dead_code)]
pub static KBD_JP_HENKAN: Kc = 136;

#[allow(dead_code)]
pub static KBD_MODIFIER_NONE: Kc = 0x00;
#[allow(dead_code)]
pub static KBD_MODIFIER_LEFT_CTRL: Kc = 0x01;
#[allow(dead_code)]
pub static KBD_MODIFIER_LEFT_SHIFT: Kc = 0x02;
#[allow(dead_code)]
pub static KBD_MODIFIER_LEFT_ALT: Kc = 0x04;
#[allow(dead_code)]
pub static KBD_MODIFIER_LEFT_UI: Kc = 0x08;
#[allow(dead_code)]
pub static KBD_MODIFIER_RIGHT_CTRL: Kc = 0x10;
#[allow(dead_code)]
pub static KBD_MODIFIER_RIGHT_SHIFT: Kc = 0x20;
#[allow(dead_code)]
pub static KBD_MODIFIER_RIGHT_ALT: Kc = 0x40;
#[allow(dead_code)]
pub static KBD_MODIFIER_RIGHT_UI: Kc = 0x80;
