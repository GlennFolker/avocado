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
