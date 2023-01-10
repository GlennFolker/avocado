use crate::core::prelude::*;
use bevy_math::Vec2;
use bevy_utils::HashMap;
use std::hash::Hash;

pub trait InputAction: 'static + Copy + PartialEq + Eq + Hash + Send + Sync {}
impl<T: 'static + Copy + PartialEq + Eq + Hash + Send + Sync> InputAction for T {}

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum InputValue {
    Single {
        pressed: bool,
        tapped: bool,
    },
    Linear(f32),
    Axis(Vec2),
}

impl InputValue {
    #[inline]
    pub fn pressed(self) -> Option<bool> {
        match self {
            Self::Single { pressed, .. } => Some(pressed),
            _ => None,
        }
    }

    #[inline]
    pub fn tapped(self) -> Option<bool> {
        match self {
            Self::Single { tapped, .. } => Some(tapped),
            _ => None,
        }
    }

    #[inline]
    pub fn linear(self) -> Option<f32> {
        match self {
            Self::Linear(linear) => Some(linear),
            _ => None,
        }
    }

    #[inline]
    pub fn axis(self) -> Option<Vec2> {
        match self {
            Self::Axis(axis) => Some(axis),
            _ => None,
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct InputState<T: InputAction> {
    pub(crate) values: HashMap<T, InputValue>,
}

impl<T: InputAction> InputState<T> {
    #[inline]
    pub fn pressed(self, key: &T) -> Option<bool> {
        self.values.get(key).map(|val| val.pressed()).unwrap_or(None)
    }

    #[inline]
    pub fn tapped(self, key: &T) -> Option<bool> {
        self.values.get(key).map(|val| val.tapped()).unwrap_or(None)
    }

    #[inline]
    pub fn linear(self, key: &T) -> Option<f32> {
        self.values.get(key).map(|val| val.linear()).unwrap_or(None)
    }

    #[inline]
    pub fn axis(self, key: &T) -> Option<Vec2> {
        self.values.get(key).map(|val| val.axis()).unwrap_or(None)
    }
}
