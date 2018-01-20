use std::borrow::Cow;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::rc::Rc;

use lunch::Lunchable;

pub trait Search {
    fn search_terms<'a>(&'a self) -> SearchTerms<'a>;
}

pub struct SearchTerms<'a> {
    pub terms: Vec<Cow<'a, str>>,
    pub keywords: Vec<Cow<'a, str>>,
    pub related: Option<Rc<Lunchable>>,
}

impl<'a> Debug for SearchTerms<'a> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "SearchTerms {{ terms: {:?}, keywords: {:?} }}", self.terms, self.keywords)
    }
}
