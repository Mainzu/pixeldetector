#[cfg(test)]
mod tests {
    use super::*;

    use std::ops::Sub;
    struct Diff<I: Iterator>
    where
        I::Item: Sub + Copy,
    {
        it: I,
        prev: Option<I::Item>,
    }
    impl<I> Diff<I>
    where
        I: Iterator,
        I::Item: Sub + Copy,
    {
        fn new(it: I) -> Self {
            Self { it, prev: None }
        }
    }

    impl<I> Iterator for Diff<I>
    where
        I: Iterator,
        I::Item: Sub + Copy,
    {
        type Item = <I::Item as Sub>::Output;

        fn next(&mut self) -> Option<Self::Item> {
            let prev = self.prev.take().or_else(|| self.it.next())?;

            let next = self.it.next()?;
            let diff = prev - next;
            self.prev = Some(next);
            Some(diff)
        }
    }

    #[test]
    fn diff_simple() {
        assert_eq!(
            Diff::new([5, 1, 2, 7].into_iter()).collect::<Vec<_>>(),
            vec![4, -1, -5]
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum IndexResult<T> {
    Empty,
    InRange(T),
    OutOfRange,
}
impl<T> IndexResult<T> {
    fn new(index: usize, length: usize, f: impl FnOnce() -> T) -> Self {
        if length == 0 {
            Self::Empty
        } else if index < length {
            Self::InRange(f())
        } else {
            Self::OutOfRange
        }
    }
    fn into_opt_res(self) -> Option<Result<T, OutOfBounds>> {
        match self {
            IndexResult::Empty => None,
            IndexResult::InRange(value) => Some(Ok(value)),
            IndexResult::OutOfRange => Some(Err(OutOfBounds)),
        }
    }

    fn unwrap(self) -> Result<T, OutOfBounds> {
        self.into_opt_res().unwrap()
    }
    fn unwrap2(self) -> T {
        self.into_opt_res().unwrap().unwrap()
    }
}
impl<T> From<IndexResult<T>> for Option<Result<T, OutOfBounds>> {
    fn from(value: IndexResult<T>) -> Self {
        value.into_opt_res()
    }
}

#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct OutOfBounds;
// pub(crate) type IndexResult<T> = Option<Result<T, OutOfBounds>>;

pub(crate) fn index_with<T>(index: usize, length: usize, f: impl FnOnce() -> T) -> IndexResult<T> {
    // if length == 0 {
    //     None
    // } else if index < length {
    //     Some(Ok(f()))
    // } else {
    //     Some(Err(OutOfBounds))
    // }
    IndexResult::new(index, length, f)
}
impl std::fmt::Display for OutOfBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "out of bounds")
    }
}
impl std::error::Error for OutOfBounds {}
