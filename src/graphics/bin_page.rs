use crate::graphics::BinKey;
use bevy_utils::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BinPageError<K: BinKey> {
    #[error("{key:?} is already registered: {rect:?}")]
    AlreadyInserted {
        key: K,
        rect: BinRect,
    },
    #[error("Not enough space; try resizing")]
    NotEnoughSpace(K),
}

pub type BinPageResult<K> = Result<(), BinPageError<K>>;

#[derive(Debug)]
pub struct BinPage<K: BinKey> {
    width: u32,
    height: u32,

    mapping: HashMap<K, usize>,
    free: Vec<BinRect>,
    placed: Vec<BinRect>,

    new_free: Vec<BinRect>,
    new_free_last: usize,
}

impl<K: BinKey> BinPage<K> {
    #[inline]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width, height,

            mapping: HashMap::default(),
            free: vec![BinRect {
                x: 0, y: 0,
                width, height,
            }],
            placed: vec![],

            new_free: vec![],
            new_free_last: 0,
        }
    }

    #[inline]
    pub fn get(&self, key: &K) -> Option<BinRect> {
        self.mapping.get(key).map(|index| self.placed[*index])
    }

    pub fn insert(&mut self, key: impl Into<K>, width: u32, height: u32) -> BinPageResult<K> {
        let key = key.into();
        if let (Some(rect), _, _) = self.find_pos(width, height) {
            if self.mapping.contains_key(&key) {
                let rect = self.placed[self.mapping[&key]];
                return Err(BinPageError::AlreadyInserted {key, rect, });
            } else {
                let index = self.place(rect);
                self.mapping.insert(key, index);
            }

            Ok(())
        } else {
            Err(BinPageError::NotEnoughSpace(key))
        }
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width <= self.width && new_height <= self.height {
            return;
        }

        let mut resized = Self::new(new_width.max(self.width), new_height.max(self.height));
        for (key, value) in self.mapping.drain() {
            let rect = self.placed[value];
            match resized.insert(key, rect.width, rect.height) {
                Ok(_) => {},
                Err(err) => log::warn!("{:?}", err),
            }
        }

        *self = resized;
    }

    #[inline]
    pub fn occupancy(&self) -> f32 {
        self.occupancy_f64() as f32
    }

    #[inline]
    pub fn occupancy_f64(&self) -> f64 {
        let mut used_area = 0;
        for placed in &self.placed {
            used_area += (placed.width as u64) * (placed.height as u64);
        }

        (used_area as f64) / ((self.width as u64) * (self.height as u64)) as f64
    }

    #[inline]
    pub fn score(&self, width: u32, height: u32) -> (u32, u32) {
        if let (Some(_), best_short_fit, best_long_fit) = self.find_pos(width, height) {
            (best_short_fit, best_long_fit)
        } else {
            (u32::MAX, u32::MAX)
        }
    }

    pub fn find_pos(&self, width: u32, height: u32) -> (Option<BinRect>, u32, u32) {
        let mut best = None;
        let mut best_short_fit = u32::MAX;
        let mut best_long_fit = u32::MAX;

        for free in self.free.iter() {
            if free.width >= width && free.height >= height {
                let leftover_hor = free.width.abs_diff(width);
                let leftover_ver = free.height.abs_diff(height);

                let short_fit = leftover_hor.min(leftover_ver);
                let long_fit = leftover_hor.max(leftover_ver);

                if short_fit < best_short_fit || (short_fit == best_short_fit && long_fit < best_long_fit) {
                    best = Some(BinRect {
                        x: free.x, y: free.y,
                        width, height,
                    });

                    best_short_fit = short_fit;
                    best_long_fit = long_fit;
                }
            }
        }

        (best, best_short_fit, best_long_fit)
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    fn place(&mut self, rect: BinRect) -> usize {
        let mut i = 0;
        while i < self.free.len() {
            let free = self.free[i];
            if
                rect.x < free.x + free.width && rect.x + rect.width > free.x &&
                rect.y < free.y + free.height && rect.y + rect.height > free.y
            {
                self.new_free_last = self.new_free.len();
                if rect.x < free.x + free.width && rect.x + rect.width > free.x {
                    if rect.y > free.y && rect.y < free.y + free.height {
                        self.insert_new(BinRect {
                            height: rect.y - free.y,
                            ..free
                        });
                    }

                    if rect.y + rect.height < free.y + free.height {
                        self.insert_new(BinRect {
                            y: rect.y + rect.height,
                            height: free.y + free.height - (rect.y + rect.height),
                            ..free
                        });
                    }
                }

                if rect.y < free.y + free.height && rect.y + rect.height > free.y {
                    if rect.x > free.x && rect.x < free.y + free.width {
                        self.insert_new(BinRect {
                            width: rect.x - free.x,
                            ..free
                        });
                    }

                    if rect.x + rect.width < free.x + free.width {
                        self.insert_new(BinRect {
                            x: rect.x + rect.width,
                            width: free.x + free.width - (rect.x + rect.width),
                            ..free
                        })
                    }
                }

                self.free[i] = self.free[self.free.len() - 1];
                self.free.pop();
            } else {
                i += 1;
            }
        }

        for free in &self.free {
            let mut i = 0;
            while i < self.new_free.len() {
                if self.new_free[i].contained_in(free) {
                    self.new_free[i] = self.new_free[self.new_free.len() - 1];
                    self.new_free.pop();
                } else {
                    i += 1;
                }
            }
        }

        self.free.append(&mut self.new_free);
        self.placed.push(rect);
        self.placed.len() - 1
    }

    fn insert_new(&mut self, rect: BinRect) {
        if rect.width == 0 || rect.height == 0 {
            return;
        }

        let mut i = 0;
        while i < self.new_free_last {
            if rect.contained_in(&self.new_free[i]) {
                return;
            }

            if self.new_free[i].contained_in(&rect) {
                self.new_free_last -= 1;

                self.new_free[i] = self.new_free[self.new_free_last];
                self.new_free[self.new_free_last] = self.new_free[self.new_free.len() - 1];
                self.new_free.pop();
            } else {
                i += 1
            }
        }

        self.new_free.push(rect);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BinRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl BinRect {
    pub fn contained_in(self, other: &BinRect) -> bool {
        self.x >= other.x && self.y >= other.y &&
        self.x + self.width <= other.x + other.width &&
        self.y + self.height <= other.y + other.height
    }
}
