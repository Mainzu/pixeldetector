use std::ops::{Add, Div};

use alg_quickselect::{get_pivot, quickselect, quickselect_unchecked};
use not_empty::NonEmptySlice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Median<T> {
    Odd(T),
    /// (Smaller, Bigger)
    Even(T, T),
}

impl<T> Median<T> {
    pub(crate) fn reduce(self, f: impl FnOnce(T, T) -> T) -> T {
        match self {
            Median::Odd(a) => a,
            Median::Even(a, b) => f(a, b),
        }
    }
    pub(crate) fn reduce_with(f: impl FnOnce(T, T) -> T) -> impl FnOnce(Self) -> T {
        move |med| med.reduce(f)
    }
    pub(crate) fn sum(self) -> T
    where
        T: Add<T, Output = T>,
    {
        self.reduce(|a, b| a + b)
    }
    pub(crate) fn get(self) -> T
    where
        T: From<u8> + Add<T, Output = T> + Div<T, Output = T>,
    {
        self.reduce(|a, b| (a + b) / T::from(2))
    }
}

pub(crate) fn get_median<T: Ord + Clone>(arr: &mut NonEmptySlice<T>) -> Median<T> {
    let k = arr.len().get() / 2;
    // let get_pivot = |s: &NonEmptySlice<T>| median_of_three::<T>(s); //|arr: &[T]| -> usize { arr.len() / 2 };

    if arr.len().get() % 2 == 0 {
        let smaller =
            unsafe { quickselect_unchecked(arr, k - 1, get_pivot::median_of_three) }.clone();
        let bigger = unsafe { quickselect_unchecked(arr, k, get_pivot::median_of_three) }.clone();
        Median::Even(smaller, bigger)
    } else {
        let median = unsafe { quickselect_unchecked(arr, k, get_pivot::median_of_three) }.clone();
        Median::Odd(median)
    }
}

// fn median_of_three<T: Ord>(arr: &[T]) -> usize {
//     let len = arr.len();
//     let mid = len / 2;
//     let first = &arr[0];
//     let middle = &arr[mid];
//     let last = &arr[len - 1];

//     if (first <= middle && middle <= last) || (last <= middle && middle <= first) {
//         mid
//     } else if (middle <= first && first <= last) || (last <= first && first <= middle) {
//         0
//     } else {
//         len - 1
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    fn quickselect<T: Ord + Debug>(arr: &mut NonEmptySlice<T>, k: usize) -> &T {
        // don't use the unchecked version, we need it to panic
        super::quickselect(arr, k, get_pivot::last_index)
    }

    fn neslice<T>(arr: &mut [T]) -> &mut NonEmptySlice<T> {
        NonEmptySlice::new_mut(arr).expect("NonEmptySlice must work correctly")
    }

    #[test]
    fn test_quickselect() {
        let mut arr = [3, 2, 1, 5, 6, 4];
        let arr = neslice(&mut arr);
        assert_eq!(quickselect(arr, 0), &1);
        assert_eq!(quickselect(arr, 1), &2);
        assert_eq!(quickselect(arr, 2), &3);
        assert_eq!(quickselect(arr, 3), &4);
        assert_eq!(quickselect(arr, 4), &5);
        assert_eq!(quickselect(arr, 5), &6);
    }

    #[test]
    #[should_panic(expected = "NonEmptySlice must work correctly")]
    fn test_quickselect_empty() {
        let mut arr: [i32; 0] = [];
        let arr = neslice(&mut arr);
    }

    #[test]
    fn test_quickselect_duplicates() {
        let mut arr = [3, 3, 3, 3, 3];
        let arr = neslice(&mut arr);
        assert_eq!(quickselect(arr, 0), &3);
        assert_eq!(quickselect(arr, 1), &3);
        assert_eq!(quickselect(arr, 2), &3);
        assert_eq!(quickselect(arr, 3), &3);
        assert_eq!(quickselect(arr, 4), &3);
    }

    #[test]
    fn test_quickselect_large() {
        let mut arr = [9, 8, 7, 6, 5, 4, 3, 2, 1];
        let arr = neslice(&mut arr);
        assert_eq!(quickselect(arr, 0), &1);
        assert_eq!(quickselect(arr, 1), &2);
        assert_eq!(quickselect(arr, 2), &3);
        assert_eq!(quickselect(arr, 3), &4);
        assert_eq!(quickselect(arr, 4), &5);
        assert_eq!(quickselect(arr, 5), &6);
        assert_eq!(quickselect(arr, 6), &7);
        assert_eq!(quickselect(arr, 7), &8);
        assert_eq!(quickselect(arr, 8), &9);
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 3 but the index is 3")]
    fn test_quickselect_bounds() {
        let mut arr = [3, 1, 2];
        let arr = neslice(&mut arr);
        quickselect(arr, 3);
    }

    #[test]
    fn range_over_bounds() {
        let a = [0, 1, 2];
        let s = a.as_slice();

        assert_eq!(s.len(), 3);

        let x = &s[3..];
        assert_eq!(x.len(), 0);

        let x = &s[..0];
        assert_eq!(x.len(), 0);
    }

    #[test]
    #[should_panic]
    fn swap_empty_array() {
        let mut a: [i32; 0] = [];
        a.swap(1, 0);
    }
    #[test]
    #[should_panic]
    fn swap_out_of_bounds_array() {
        let mut a: [i32; 1] = [1];
        a.swap(0, 1);
    }
}
