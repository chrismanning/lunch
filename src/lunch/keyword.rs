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
            search_item
                .search_terms()
                .keywords
                .iter()
                .any(|k| k == keyword)
        }) {
            return Some(self.search_items.swap_remove(n));
        }
        if keyword.len() > 3 {
            if let Some(n) = self.find(|ref search_item| {
                search_item
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
            .filter(|&(_, ref search_item)| predicate(search_item.as_ref()))
            .map(|(n, _)| n)
            .next()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        // TODO test keyword search

        assert!(false)
    }
}
