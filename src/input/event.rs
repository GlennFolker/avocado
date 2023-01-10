use crate::input::KeyCode;

pub struct KeyEvent {
    /// `true` if pressed, `false` if released.
    pub pressed: bool,
    pub key: KeyCode,
}

pub struct KeyModifierEvent {
    pub alt: bool,
    pub ctrl: bool,
    pub logo: bool,
    pub shift: bool,
}
