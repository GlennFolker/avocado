use crate::{
    core::prelude::*,
    input::{
        KeyCode,
        InputAction, InputValue,
    },
};
use bevy_math::Vec2;
use bevy_utils::{
    HashMap, HashSet
};
use smallvec::SmallVec;
use std::iter;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum InputBinding {
    Single(SmallVec<[KeyCode; 2]>),
    Linear(SmallVec<[InputLinear; 2]>),
    Axis(SmallVec<[InputAxis; 2]>),
}

impl InputBinding {
    #[inline]
    pub fn single(keys: &[KeyCode]) -> Self {
        Self::Single(SmallVec::from_slice(keys))
    }

    #[inline]
    pub fn linear(linear: &[InputLinear]) -> Self {
        Self::Linear(SmallVec::from_slice(linear))
    }

    #[inline]
    pub fn axis(axis: &[InputAxis]) -> Self {
        Self::Axis(SmallVec::from_slice(axis))
    }

    pub fn keys(&self) -> Vec<KeyCode> {
        match self {
            Self::Single(single) => single.iter().copied().collect(),
            Self::Linear(linear) => linear.iter().flat_map(|i|
                iter::once(i.positive)
                .chain(iter::once(i.negative))
            ).collect(),
            Self::Axis(axis) => axis.iter().flat_map(|i| match i {
                InputAxis::Keys { up, down, left, right, } =>
                    iter::once(*up)
                    .chain(iter::once(*down))
                    .chain(iter::once(*left))
                    .chain(iter::once(*right))
            }).collect(),
        }
    }

    pub fn value(
        &self,
        key_down: &HashSet<KeyCode>, key_tapped: &HashSet<KeyCode>,
    ) -> InputValue {
        match self {
            Self::Single(single) => {
                for input in single {
                    if key_down.contains(input) {
                        return InputValue::Single {
                            pressed: true,
                            tapped: key_tapped.contains(input),
                        };
                    }
                }

                InputValue::Single {
                    pressed: false,
                    tapped: false,
                }
            },
            Self::Linear(linear) => {
                for input in linear {
                    let add = key_down.contains(&input.positive);
                    let sub = key_down.contains(&input.negative);
                    if add || sub {
                        return InputValue::Linear(
                            (if add { 1. } else { 0. }) -
                            (if sub { 1. } else { 0. })
                        )
                    }
                }

                InputValue::Linear(0.)
            },
            Self::Axis(axis) => {
                for input in axis {
                    match input {
                        InputAxis::Keys { up, down, left, right, } => {
                            let up = key_down.contains(up);
                            let down = key_down.contains(down);
                            let left = key_down.contains(left);
                            let right = key_down.contains(right);

                            if up || down || left || right {
                                return InputValue::Axis(Vec2 {
                                    x:
                                        (if right { 1. } else { 0. }) -
                                        (if left { 1. } else { 0. }),
                                    y:
                                        (if up { 1. } else { 0. }) -
                                        (if down { 1. } else { 0. }),
                                }.normalize())
                            }
                        }
                    }
                }

                InputValue::Axis(Vec2::splat(0.))
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InputLinear {
    pub positive: KeyCode,
    pub negative: KeyCode,
}

impl InputLinear {
    #[inline]
    pub fn new(positive: KeyCode, negative: KeyCode) -> Self {
        Self { positive, negative, }
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum InputAxis {
    Keys {
        up: KeyCode,
        down: KeyCode,
        left: KeyCode,
        right: KeyCode,
    },
}

impl InputAxis {
    #[inline]
    pub fn keys(up: KeyCode, down: KeyCode, left: KeyCode, right: KeyCode) -> Self {
        Self::Keys { up, down, left, right }
    }
}

#[derive(Resource)]
pub struct InputBindings<T: InputAction> {
    pub(crate) map: HashMap<T, InputBinding>,
}

impl<T: InputAction> Default for InputBindings<T> {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
        }
    }
}
