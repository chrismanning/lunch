use super::env::Lunchable;

pub struct Keyword {
    lunchables: Vec<Box<Lunchable>>,
}

impl Keyword {
    pub fn new(lunchables: Vec<Box<Lunchable>>) -> Self {
        Keyword { lunchables }
    }

    pub fn search(mut self, keyword: &str) -> Option<Box<Lunchable>> {
        if let Some((n, _)) = self.find(|ref lunchable| {
            lunchable
                .search_terms()
                .keywords
                .iter()
                .any(|k| k == keyword)
        }) {
            return Some(self.lunchables.swap_remove(n));
        }
        if keyword.len() > 3 {
            if let Some((n, _)) = self.find(|ref lunchable| {
                lunchable
                    .search_terms()
                    .keywords
                    .iter()
                    .any(|k| k.starts_with(keyword))
            }) {
                return Some(self.lunchables.swap_remove(n));
            }
        }
        None
    }

    fn find<P>(&self, mut predicate: P) -> Option<(usize, &Box<Lunchable>)>
    where
        P: FnMut(&Box<Lunchable>) -> bool,
    {
        self.lunchables
            .iter()
            .enumerate()
            .filter(|&(_, ref lunchable)| predicate(lunchable))
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
