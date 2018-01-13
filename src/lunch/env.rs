use std::fmt::{Display, Formatter, Result as FmtResult};
use std::rc::Rc;

use super::errors::*;

use super::Launch;
use super::{Search, SearchTerms};
use super::keyword::Keyword;

use super::freedesktop::env::init_lunch;

pub trait Lunchable: Launch + Search + Display {}

impl<T> Lunchable for T
where
    T: Launch + Search + Display,
{
}

pub struct LunchEnv {
    pub lunchables: Vec<Rc<Lunchable>>,
}

impl LunchEnv {
    pub fn init() -> Result<Self> {
        init_lunch()
    }

    pub fn keyword(self, keyword: &str) -> Option<Rc<Lunchable>> {
        info!("Searching for keyword '{}'", keyword);
        let k = Keyword::<_, Lunchable>::new(self.lunchables);
        k.search(keyword)
    }

    pub fn search<Terms, S>(self, terms: Terms) -> Option<Box<Lunchable>>
    where
        Terms: Iterator<Item = S>,
        S: AsRef<str>,
    {
        for term in terms {
            info!("Searching for term '{}'", term.as_ref());
        }
        unimplemented!()
    }
}
