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
