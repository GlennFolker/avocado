use crate::{
    graphics::prelude::*,
    input::prelude::*,
};

use winit::event::VirtualKeyCode as Key;

pub trait ColorExt: Sized {
    fn from_wgpu(other: wgpu::Color) -> Self;
    fn into_wgpu(self) -> wgpu::Color;
}

impl ColorExt for Color {
    #[inline]
    fn from_wgpu(other: wgpu::Color) -> Self {
        Self::rgba(other.r as f32, other.g as f32, other.b as f32, other.a as f32)
    }

    #[inline]
    fn into_wgpu(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

pub trait KeyCodeExt: Sized {
    fn from_vkey(key: Key) -> Option<Self>;
}

impl KeyCodeExt for KeyCode {
    fn from_vkey(key: Key) -> Option<Self> {
        use KeyCode::*;
        Some(match key {
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

            _ => return None,
        })
    }
}
