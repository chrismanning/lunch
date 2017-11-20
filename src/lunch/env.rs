use std::fmt::Display;

use super::errors::*;

use super::Launch;
use super::Search;
use super::keyword::Keyword;

use super::freedesktop::env::FreeDesktopEnv as PlatformEnv;

pub trait Lunchable: Launch + Search + Display {}

impl<T> Lunchable for T
where
    T: Launch + Search + Display,
{
}

pub struct LunchEnv {
    lunchables: Vec<Box<Lunchable>>,
}

impl LunchEnv {
    pub fn init() -> Result<Self> {
        PlatformEnv::init_lunch()
    }

    pub fn keyword(self, keyword: &str) -> Option<Box<Lunchable>> {
        let k = Keyword::new(self.lunchables);
        k.search(keyword)
    }
}
