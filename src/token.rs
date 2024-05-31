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
pub struct Modifier(pub String);
#[derive(Debug, Clone)]
pub struct Key {
    pub key: String,
    pub attribute: KeyAttribute,
}
