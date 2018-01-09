use std::fmt::{Display, Formatter, Result as FmtResult};
use std::rc::Rc;

use super::errors::*;

use super::Launch;
use super::{Search, SearchTerms};
use super::keyword::Keyword;

use super::freedesktop::env::FreeDesktopEnv as PlatformEnv;

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
        PlatformEnv::init_lunch()
    }

    pub fn keyword(self, keyword: &str) -> Option<Rc<Lunchable>> {
        info!("Searching for keyword '{}'", keyword);
        let k = Keyword::<_, Lunchable>::new(self.lunchables);
        k.search(keyword)
    }

    pub fn search<Terms, S>(self, terms: Terms) -> Option<Box<Lunchable>>
    where
        Terms: Iterator<Item=S>,
        S: AsRef<str>,
    {
        for term in terms {
            info!("Searching for term '{}'", term.as_ref());
        }
        unimplemented!()
    }
}

pub struct BasicLunchable {
    pub launch: Rc<Launch>,
    pub search: Rc<Search>,
    pub display: Rc<Display>,
}

impl BasicLunchable {

}

impl Launch for BasicLunchable {
    fn launch(&self, args: Vec<String>) -> Error {
        self.launch.launch(args)
    }
}

impl Search for BasicLunchable {
    fn search_terms<'a>(&'a self) -> SearchTerms<'a> {
        self.search.search_terms()
    }
}

impl Display for BasicLunchable {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.display.fmt(f)
    }
}
