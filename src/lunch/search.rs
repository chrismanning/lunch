use std::borrow::Cow;

pub trait Search {
    fn search_terms<'a>(&'a self) -> SearchTerms<'a>;
}

pub struct SearchTerms<'a> {
    pub terms: Vec<Cow<'a, str>>,
    pub keywords: Vec<Cow<'a, str>>,
}
