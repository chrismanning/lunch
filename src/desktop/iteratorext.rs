use std::iter::Iterator;
use std::result::Result;

pub trait IteratorExt {
    fn filter_result<Pred, T, E>(self, pred: Pred) -> FilterResult<Self, Pred>
    where
        Self: Iterator<Item = Result<T, E>> + Sized,
        Pred: FnMut(&T) -> bool,
    {
        FilterResult {
            iterator: self,
            predicate: pred,
        }
    }

    fn skip_while_result<Pred, T, E>(self, pred: Pred) -> SkipWhileResult<Self, Pred>
    where
        Self: Iterator<Item = Result<T, E>> + Sized,
        Pred: FnMut(&T) -> bool,
    {
        SkipWhileResult {
            iterator: self,
            predicate: pred,
            flag: false,
        }
    }

    fn take_while_result<Pred, T, E>(self, pred: Pred) -> TakeWhileResult<Self, Pred>
    where
        Self: Iterator<Item = Result<T, E>> + Sized,
        Pred: FnMut(&T) -> bool,
    {
        TakeWhileResult {
            iterator: self,
            predicate: pred,
            flag: false,
        }
    }
}

impl<Iter: ?Sized> IteratorExt for Iter
where
    Iter: Iterator,
{
}

pub struct FilterResult<Iter, Pred> {
    iterator: Iter,
    predicate: Pred,
}

impl<Iter, T, E, Pred> Iterator for FilterResult<Iter, Pred>
where
    Iter: Iterator<Item = Result<T, E>>,
    Pred: FnMut(&T) -> bool,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        for result in self.iterator.by_ref() {
            match result {
                Ok(t) => {
                    if (self.predicate)(&t) {
                        return Some(Ok(t));
                    }
                }
                Err(err) => return Some(Err(err)),
            }
        }
        None
    }
}

pub struct TakeWhileResult<Iter, Pred> {
    iterator: Iter,
    predicate: Pred,
    flag: bool,
}

impl<Iter, T, E, Pred> Iterator for TakeWhileResult<Iter, Pred>
where
    Iter: Iterator<Item = Result<T, E>>,
    Pred: FnMut(&T) -> bool,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.flag {
            None
        } else {
            self.iterator.next().and_then(|x| match x {
                Ok(x) => {
                    if (self.predicate)(&x) {
                        Some(Ok(x))
                    } else {
                        self.flag = true;
                        None
                    }
                }
                Err(err) => Some(Err(err)),
            })
        }
    }
}

pub struct SkipWhileResult<Iter, Pred> {
    iterator: Iter,
    predicate: Pred,
    flag: bool,
}

impl<Iter, T, E, Pred> Iterator for SkipWhileResult<Iter, Pred>
where
    Iter: Iterator<Item = Result<T, E>>,
    Pred: FnMut(&T) -> bool,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        for x in self.iterator.by_ref() {
            match x {
                Ok(x) => {
                    self.flag = !(self.predicate)(&x);
                    if self.flag {
                        return Some(Ok(x));
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use std::iter::Iterator;
    use super::*;

    #[test]
    pub fn filter_result() {
        let input = vec![Ok(1), Ok(2), Ok(3), Ok(4), Err("er")];
        let res: Vec<_> = input
            .into_iter()
            .filter_result(|x| *x < 2 || *x > 3)
            .collect();
        assert_eq!(res, vec![Ok(1), Ok(4), Err("er")]);
    }

    #[test]
    pub fn take_while_result() {
        let input = vec![Ok(1), Ok(2), Err("er"), Ok(3), Ok(4), Err("er")];
        let res: Vec<_> = input.into_iter().take_while_result(|x| *x < 4).collect();
        assert_eq!(res, vec![Ok(1), Ok(2), Err("er"), Ok(3)]);
    }

    #[test]
    pub fn skip_while_result() {
        let input = vec![Ok(1), Ok(2), Err("er"), Ok(3), Ok(4), Err("er")];
        let res: Vec<_> = input.into_iter().skip_while_result(|x| *x < 4).collect();
        assert_eq!(res, vec![Err("er"), Ok(4), Err("er")]);
    }
}
