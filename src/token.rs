bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct KeyAttribute: u8 {
        const None = 0b00000000;
        const Send = 0b00000001;
        const OnRelease = 0b00000010;
        const Both = Self::Send.bits() | Self::OnRelease.bits();
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Modifier(Modifier),
    Key(Key),
}

#[derive(Debug, Clone)]
pub struct Modifier(String);
#[derive(Debug, Clone)]
pub struct Key {
    pub key: String,
    pub attribute: KeyAttribute,
}

impl Token {
    pub fn new_key(key: String, attribute: KeyAttribute) -> Self {
        Self::Key(Key { key, attribute })
    }
    pub fn new_key_from_string(key: String) -> Self {
        Self::Key(Key {
            key,
            attribute: KeyAttribute::None,
        })
    }
    pub fn new_modifier(modifier: String) -> Self {
        Self::Modifier(Modifier(modifier))
    }
}
