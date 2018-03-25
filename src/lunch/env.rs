use std::fmt::Display;
use std::rc::Rc;

use super::errors::*;

use super::Launch;
use super::SearchIdxItem;
use super::keyword::Keyword;
use super::search::Searcher;

use super::freedesktop::env::init_lunch;

pub trait Lunchable: Launch + SearchIdxItem + Display {}

impl<T> Lunchable for T
where
    T: Launch + SearchIdxItem + Display,
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
        S: ::std::fmt::Debug,
    {
        let terms: Vec<_> = terms.collect();
        let searcher = Searcher {
            lunchables: self.lunchables,
        };
        
        searcher.score(terms);
        // TODO scored search
        // each term match contributes to score total
        // terms are concat'd
        // eg. ["a","b","c"] => [["a","b","c"], ["a b","c"], ["a b c"], ["a", "b c"]]
        // keyword match with other terms = high score
        // keyword match with single term = exact match
        // different scores for: exact match, approx match, starts with, contains, etc.
        unimplemented!()
    }
}
