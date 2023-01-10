use crate::{
    core::prelude::*,
    input::KeyModifierEvent,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    N1, N2, N3, N4, N5,
    N6, N7, N8, N9, N0,

    A, B, C, D, E, F, G,
    H, I, J, K, L, M, N,
    O, P, Q, R, S, T,
    U, V, W, X, Y, Z,

    F1, F2, F3, F4, F5, F6,
    F7, F8, F9, F10, F11, F12,
}

impl KeyCode {
    #[inline]
    pub fn is_num(self) -> bool {
        use KeyCode::*;
        match self {
            N1 | N2 | N3 | N4 | N5 |
            N6 | N7 | N8 | N9 | N0 => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_alphabet(self) -> bool {
        use KeyCode::*;
        match self {
            A | B | C | D | E | F | G |
            H | I | J | K | L | M | N |
            O | P | Q | R | S | T |
            U | V | W | X | Y | Z => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_fn(self) -> bool {
        use KeyCode::*;
        match self {
            F1 | F2 | F3 | F4 | F5 | F6 |
            F7 | F8 | F9 | F10 | F11 | F12 => true,
            _ => false,
        }
    }
}

#[derive(Resource, Default)]
pub struct KeyModifier {
    alt: bool,
    ctrl: bool,
    logo: bool,
    shift: bool,
}

impl KeyModifier {
    pub fn update_sys(mut modifier: ResMut<Self>, mut events: EventReader<KeyModifierEvent>) {
        if !events.is_empty() {
            let event = events.iter().next_back().unwrap();
            modifier.alt = event.alt;
            modifier.ctrl = event.ctrl;
            modifier.logo = event.logo;
            modifier.shift = event.shift;

            events.clear();
        }
    }

    #[inline]
    pub fn alt(&self) -> bool {
        self.alt
    }

    #[inline]
    pub fn ctrl(&self) -> bool {
        self.ctrl
    }

    #[inline]
    pub fn logo(&self) -> bool {
        self.logo
    }

    #[inline]
    pub fn shift(&self) -> bool {
        self.shift
    }
}
