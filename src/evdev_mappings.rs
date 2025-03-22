use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::ParseError;
use evdev::Key;

// lazy_static to initialize a static global HashMap
lazy_static! {
    static ref KEY_MAP: HashMap<&'static str, Key> = {
        let mut m = HashMap::new();
        m.insert("q", Key::KEY_Q);
        m.insert("w", Key::KEY_W);
        m.insert("e", Key::KEY_E);
        m.insert("r", Key::KEY_R);
        m.insert("t", Key::KEY_T);
        m.insert("y", Key::KEY_Y);
        m.insert("u", Key::KEY_U);
        m.insert("i", Key::KEY_I);
        m.insert("o", Key::KEY_O);
        m.insert("p", Key::KEY_P);
        m.insert("a", Key::KEY_A);
        m.insert("s", Key::KEY_S);
        m.insert("d", Key::KEY_D);
        m.insert("f", Key::KEY_F);
        m.insert("g", Key::KEY_G);
        m.insert("h", Key::KEY_H);
        m.insert("j", Key::KEY_J);
        m.insert("k", Key::KEY_K);
        m.insert("l", Key::KEY_L);
        m.insert("z", Key::KEY_Z);
        m.insert("x", Key::KEY_X);
        m.insert("c", Key::KEY_C);
        m.insert("v", Key::KEY_V);
        m.insert("b", Key::KEY_B);
        m.insert("n", Key::KEY_N);
        m.insert("m", Key::KEY_M);
        m.insert("1", Key::KEY_1);
        m.insert("2", Key::KEY_2);
        m.insert("3", Key::KEY_3);
        m.insert("4", Key::KEY_4);
        m.insert("5", Key::KEY_5);
        m.insert("6", Key::KEY_6);
        m.insert("7", Key::KEY_7);
        m.insert("8", Key::KEY_8);
        m.insert("9", Key::KEY_9);
        m.insert("0", Key::KEY_0);
        m.insert("escape", Key::KEY_ESC);
        m.insert("backspace", Key::KEY_BACKSPACE);
        m.insert("capslock", Key::KEY_CAPSLOCK);
        m.insert("return", Key::KEY_ENTER);
        m.insert("enter", Key::KEY_ENTER);
        m.insert("tab", Key::KEY_TAB);
        m.insert("space", Key::KEY_SPACE);
        m.insert("plus", Key::KEY_KPPLUS);
        m.insert("minus", Key::KEY_MINUS);
        m.insert("-", Key::KEY_MINUS);
        m.insert("equal", Key::KEY_EQUAL);
        m.insert("=", Key::KEY_EQUAL);
        m.insert("grave", Key::KEY_GRAVE);
        m.insert("`", Key::KEY_GRAVE);
        m.insert("print", Key::KEY_SYSRQ);
        m.insert("volumeup", Key::KEY_VOLUMEUP);
        m.insert("volumedown", Key::KEY_VOLUMEDOWN);
        m.insert("mute", Key::KEY_MUTE);
        m.insert("brightnessup", Key::KEY_BRIGHTNESSUP);
        m.insert("brightnessdown", Key::KEY_BRIGHTNESSDOWN);
        m.insert("comma", Key::KEY_COMMA);
        m.insert(",", Key::KEY_COMMA);
        m.insert("dot", Key::KEY_DOT);
        m.insert("period", Key::KEY_DOT);
        m.insert(".", Key::KEY_DOT);
        m.insert("slash", Key::KEY_SLASH);
        m.insert("/", Key::KEY_SLASH);
        m.insert("backslash", Key::KEY_BACKSLASH);
        m.insert("\\", Key::KEY_BACKSLASH);
        m.insert("leftbrace", Key::KEY_LEFTBRACE);
        m.insert("[", Key::KEY_LEFTBRACE);
        m.insert("rightbrace", Key::KEY_RIGHTBRACE);
        m.insert("]", Key::KEY_RIGHTBRACE);
        m.insert("semicolon", Key::KEY_SEMICOLON);
        m.insert(";", Key::KEY_SEMICOLON);
        m.insert("apostrophe", Key::KEY_APOSTROPHE);
        m.insert("'", Key::KEY_APOSTROPHE);
        m.insert("left", Key::KEY_LEFT);
        m.insert("right", Key::KEY_RIGHT);
        m.insert("up", Key::KEY_UP);
        m.insert("down", Key::KEY_DOWN);
        m.insert("pause", Key::KEY_PAUSE);
        m.insert("home", Key::KEY_HOME);
        m.insert("delete", Key::KEY_DELETE);
        m.insert("insert", Key::KEY_INSERT);
        m.insert("end", Key::KEY_END);
        m.insert("pagedown", Key::KEY_PAGEDOWN);
        m.insert("pageup", Key::KEY_PAGEUP);
        m.insert("f1", Key::KEY_F1);
        m.insert("f2", Key::KEY_F2);
        m.insert("f3", Key::KEY_F3);
        m.insert("f4", Key::KEY_F4);
        m.insert("f5", Key::KEY_F5);
        m.insert("f6", Key::KEY_F6);
        m.insert("f7", Key::KEY_F7);
        m.insert("f8", Key::KEY_F8);
        m.insert("f9", Key::KEY_F9);
        m.insert("f10", Key::KEY_F10);
        m.insert("f11", Key::KEY_F11);
        m.insert("f12", Key::KEY_F12);
        m
    };
}

pub fn convert(s: &str) -> Result<Key, ParseError> {
    KEY_MAP
        .get(s)
        .copied()
        .ok_or_else(|| ParseError::InvalidKey(s.to_string()))
}
