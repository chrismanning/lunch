use super::search::Search;

pub struct Keyword<T: Search + ?Sized> {
    search_items: Vec<Box<T>>,
}

impl<T: Search + ?Sized> Keyword<T> {
    pub fn new(search_items: Vec<Box<T>>) -> Self {
        Keyword { search_items }
    }

    pub fn search(mut self, keyword: &str) -> Option<Box<T>> {
        if let Some(n) = self.find(|ref search_item| {
            search_item.search_terms().keywords.iter().any(
                |k| k == keyword,
            )
        })
        {
            return Some(self.search_items.swap_remove(n));
        }
        if keyword.len() > 3 {
            if let Some(n) = self.find(|ref search_item| {
                search_item.search_terms().keywords.iter().any(|k| {
                    k.starts_with(keyword)
                })
            })
            {
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
            .filter(|&(_, ref search_item)| predicate(search_item.as_ref()))
            .map(|(n, _)| n)
            .next()
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use lunch::search::SearchTerms;
    use super::*;

    struct DummySearch<'a> {
        search_terms: SearchTerms<'a>,
    }

    impl<'a> DummySearch<'a> {
        fn new<'b: 'a>(terms: Vec<&'b str>, keywords: Vec<&'b str>) -> Self {
            DummySearch {
                search_terms: SearchTerms {
                    terms: terms.iter().map(|s| Cow::Borrowed(*s)).collect(),
                    keywords: keywords.iter().map(|s| Cow::Borrowed(*s)).collect(),
                },
            }
        }
    }

    impl<'a> Search for DummySearch<'a> {
        fn search_terms<'b>(&'b self) -> SearchTerms<'b> {
            SearchTerms {
                terms: self.search_terms.terms.clone(),
                keywords: self.search_terms.keywords.clone(),
            }
        }
    }

    #[test]
    fn keyword_match() {
        let keyword_searcher =
            Keyword { search_items: vec![Box::new(DummySearch::new(vec![], vec!["keyword"]))] };

        assert!(keyword_searcher.search("keyword").is_some());
    }

    #[test]
    fn keyword_contains() {
        let keyword_searcher =
            Keyword { search_items: vec![Box::new(DummySearch::new(vec![], vec!["keyword"]))] };

        assert!(keyword_searcher.search("keywor").is_some());
    }

    #[test]
    fn keyword_no_match() {
        let keyword_searcher =
            Keyword { search_items: vec![Box::new(DummySearch::new(vec![], vec!["keyword"]))] };

        assert!(keyword_searcher.search("keyword1").is_none());
    }

    #[test]
    fn keyword_too_short() {
        let keyword_searcher =
            Keyword { search_items: vec![Box::new(DummySearch::new(vec![], vec!["keyword"]))] };

        assert!(keyword_searcher.search("ke").is_none());
    }
}
