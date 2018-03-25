use std::borrow::Borrow;

use super::search::{Search, SearchIdxItem, Weight};

pub struct Keyword<T, S: ?Sized> {
    search_items: Vec<T>,
    phantom: ::std::marker::PhantomData<*mut S>,
}

impl<T, S> Keyword<T, S>
where
    S: SearchIdxItem + ?Sized,
    T: Borrow<S>,
{
    pub fn new(search_items: Vec<T>) -> Self {
        Keyword {
            search_items,
            phantom: ::std::marker::PhantomData,
        }
    }

    pub fn search(mut self, keyword: &str) -> Option<T> {
        if let Some(n) = self.find(|ref search_item: &T| {
            let search_item: &T = search_item;
            search_item
                .borrow()
                .search_terms()
                .keywords
                .iter()
                .any(|k| k == keyword)
        }) {
            return Some(self.search_items.swap_remove(n));
        }
        if keyword.len() > 3 {
            if let Some(n) = self.find(|ref search_item| {
                let search_item: &T = search_item;
                search_item
                    .borrow()
                    .search_terms()
                    .keywords
                    .iter()
                    .any(|k| k.starts_with(keyword))
            }) {
                return Some(self.search_items.swap_remove(n));
            }
        }
        None
    }

    fn find<P>(&self, mut predicate: P) -> Option<usize>
    where
        P: FnMut(&T) -> bool,
    {
        self.search_items
            .iter()
            .enumerate()
            .filter(|&(_, ref search_item)| predicate(search_item.borrow()))
            .map(|(n, _)| n)
            .next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;
    use std::borrow::Cow;
    use lunch::search::SearchIdxData;

    #[derive(Debug)]
    struct DummySearch<'a> {
        search_terms: SearchIdxData<'a>,
    }

    impl<'a> DummySearch<'a> {
        fn new<'b: 'a>(terms: Vec<&'b str>, keywords: Vec<&'b str>) -> Self {
            DummySearch {
                search_terms: SearchIdxData {
                    terms: terms.iter().map(|s| Cow::Borrowed(*s)).collect(),
                    keywords: keywords.iter().map(|s| Cow::Borrowed(*s)).collect(),
                },
            }
        }
    }

    impl<'a> SearchIdxItem for DummySearch<'a> {
        fn search_terms<'b>(&'b self) -> SearchIdxData<'b> {
            SearchIdxData {
                terms: self.search_terms.terms.clone(),
                keywords: self.search_terms.keywords.clone(),
            }
        }
    }

    #[test]
    fn keyword_match() {
        let keyword_searcher = Keyword::<Box<DummySearch>, DummySearch>::new(vec![
            Box::new(DummySearch::new(vec![], vec!["keyword"])),
        ]);

        assert_that!(keyword_searcher.search("keyword")).is_some();
    }

    #[test]
    fn keyword_contains() {
        let keyword_searcher = Keyword::<Box<DummySearch>, DummySearch>::new(vec![
            Box::new(DummySearch::new(vec![], vec!["keyword"])),
        ]);

        assert_that!(keyword_searcher.search("keywor")).is_some();
    }

    #[test]
    fn keyword_no_match() {
        let keyword_searcher = Keyword::<Box<DummySearch>, DummySearch>::new(vec![
            Box::new(DummySearch::new(vec![], vec!["keyword"])),
        ]);

        assert_that!(keyword_searcher.search("keyword1")).is_none();
    }

    #[test]
    fn keyword_too_short() {
        let keyword_searcher = Keyword::<Box<DummySearch>, DummySearch>::new(vec![
            Box::new(DummySearch::new(vec![], vec!["keyword"])),
        ]);

        assert_that!(keyword_searcher.search("ke")).is_none();
    }
}

impl<T, S> Search for Keyword<T, S> {
    fn search<I: IntoIterator<Item=SRef>, SRef: AsRef<str>>(&self, search_terms: I) -> Weight {
        unimplemented!()
    }
}
