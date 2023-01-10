use crate::graphics::{
    BinPage, BinPageError,
};
use bevy_utils::HashMap;
use thiserror::Error;
use std::{
    fmt::Debug,
    hash::Hash
};

pub trait BinKey: Eq + Hash + Debug {}
impl<T: Eq + Hash + Debug> BinKey for T {}

#[derive(Debug, Error)]
pub enum BinPackError<K: BinKey> {
    #[error("{key:?} couldn't fit: ({width}, {height}) > ({max_width}, {max_height})")]
    TooLarge {
        key: K,
        width: u32,
        height: u32,
        max_width: u32,
        max_height: u32,
    },
    #[error("Group is not registered")]
    GroupNotRegistered,
    #[error("Page error: {0:?}")]
    PageError(#[from] BinPageError<K>),
}

pub type BinPackResult<K> = Result<(), BinPackError<K>>;

#[derive(Debug)]
pub struct BinPack<G: BinKey, K: BinKey> {
    min_width: u32,
    min_height: u32,
    max_width: u32,
    max_height: u32,
    pages: HashMap<G, Vec<BinPage<K>>>,
}

impl<G: BinKey, K: BinKey> BinPack<G, K> {
    #[inline]
    pub fn new(min_width: u32, min_height: u32, max_width: u32, max_height: u32) -> Self {
        Self {
            min_width, min_height,
            max_width, max_height,
            pages: HashMap::default(),
        }
    }

    pub fn group(&mut self, group_key: impl Into<G>) -> bool {
        let group_key = group_key.into();
        if !self.pages.contains_key(&group_key) {
            self.pages.insert(group_key, vec![BinPage::new(self.min_width, self.min_height)]);
            true
        } else {
            false
        }
    }

    pub fn insert(&mut self, group: &G, key: impl Into<K>, width: u32, height: u32) -> BinPackResult<K> {
        let key = key.into();
        if width > self.max_width || height > self.max_height {
            return Err(BinPackError::TooLarge {
                key, width, height,
                max_width: self.max_width,
                max_height: self.max_height,
            });
        }

        let pages = self.pages.get_mut(group).ok_or(BinPackError::GroupNotRegistered)?;
        let (mut best_short_fit, mut best_long_fit) = (u32::MAX, u32::MAX);

        let mut best_page = None;
        for i in 0..pages.len() {
            let page = &pages[i];
            let (bin_short_fit, bin_long_fit) = page.score(width, height);

            if bin_short_fit == u32::MAX || bin_long_fit == u32::MAX {
                continue;
            }

            if best_short_fit > bin_short_fit && best_long_fit > bin_long_fit {
                best_short_fit = bin_short_fit;
                best_long_fit = bin_long_fit;
                best_page = Some(i);
            }
        }

        if let Some(best_page) = best_page {
            pages[best_page].insert(key, width, height)?;
            Ok(())
        } else {
            if let Some(key) = {
                let last = pages.len() - 1;
                let last = &mut pages[last];
                let last_w = last.width();
                let last_h = last.height();

                let returned;
                if last_w >= self.max_width && last_h >= self.max_height {
                    returned = Some(key);
                } else {
                    if last_h > last_w {
                        last.resize(
                            (last_w + last_w.max(width)).min(self.max_width),
                            (last_h.max(height)).min(self.max_height),
                        );
                    } else {
                        last.resize(
                            (last_w.max(width)).min(self.max_width),
                            (last_h + last_h.max(height)).min(self.max_height),
                        );
                    }

                    returned = match last.insert(key, width, height) {
                        Ok(_) => None,
                        Err(BinPageError::AlreadyInserted {
                            key, rect,
                        }) => return Err(BinPageError::AlreadyInserted { key, rect, }.into()),
                        Err(BinPageError::NotEnoughSpace(key)) => Some(key),
                    };
                }

                returned
            } {
                pages.push(BinPage::new(width.max(self.min_width), height.max(self.min_height)));
                let last = pages.len() - 1;
                let last = &mut pages[last];
                last.insert(key, width, height)?;
            }

            Ok(())
        }
    }

    pub fn finish(self) -> HashMap<G, Vec<BinPage<K>>> {
        self.pages
    }
}
