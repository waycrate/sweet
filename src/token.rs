bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct KeyAttribute: u8 {
        const None = 0b00000000;
        const Send = 0b00000001;
        const OnRelease = 0b00000010;
        const Both = Self::Send.bits() | Self::OnRelease.bits();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modifier(pub String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    pub key: String,
    pub attribute: KeyAttribute,
}

impl Key {
    pub fn new<S: AsRef<str>>(key: S, attribute: KeyAttribute) -> Self {
        Self {
            key: key.as_ref().to_owned(),
            attribute,
        }
    }
}
