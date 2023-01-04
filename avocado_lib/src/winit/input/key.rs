use crate::incl::{
    *,
    winit::VirtualKeyCode as Key,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    N1, N2, N3, N4, N5,
    N6, N7, N8, N9, N0,

    A, B, C, D, E, F, G,
    H, I, J, K, L, M, N,
    O, P, Q, R, S, T,
    U, V, W, X, Y, Z,
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
}

impl TryFrom<winit::VirtualKeyCode> for KeyCode {
    type Error = String;

    #[inline]
    fn try_from(key: Key) -> Result<Self, Self::Error> {
        use KeyCode::*;
        Ok(match key {
            Key::Key1 => N1,
            Key::Key2 => N2,
            Key::Key3 => N3,
            Key::Key4 => N4,
            Key::Key5 => N5,
            Key::Key6 => N6,
            Key::Key7 => N7,
            Key::Key8 => N8,
            Key::Key9 => N9,
            Key::Key0 => N0,

            Key::A => A,
            Key::B => B,
            Key::C => C,
            Key::D => D,
            Key::E => E,
            Key::F => F,
            Key::G => G,
            Key::H => H,
            Key::I => I,
            Key::J => J,
            Key::K => K,
            Key::L => L,
            Key::M => M,
            Key::N => N,
            Key::O => O,
            Key::P => P,
            Key::Q => Q,
            Key::R => R,
            Key::S => S,
            Key::T => T,
            Key::U => U,
            Key::V => V,
            Key::W => W,
            Key::X => X,
            Key::Y => Y,
            Key::Z => Z,

            _ => return Err(format!("Key {:?} unimplemented", key)),
        })
    }
}
