#![feature(slice_swap_unchecked)]

use std::ops::Sub;

use anyhow::{Ok, Result};

use image::imageops::thumbnail;
use image::io::Reader as ImageReader;
use ndarray;
use ndarray::{prelude::*, Slice};
use not_empty::NonEmptyVec;
use nshare::RefNdarray3;
use num_integer::gcd;
use ordered_float::NotNan;

mod utils;

// /// It is recommened to use [`quick_select_val`] instead if T implement [`Copy`] and
// /// the [size of](std::mem::size_of()) `T` is less than the size of `&T`.
// fn quick_select_ref<'a, T: Ord>(
//     x: &[&'a T],
//     k: usize,
//     mut get_pivot: impl for<'b> FnMut(&[&'b T]) -> &'b T,
// ) -> &'a T {
//     if x.len() == 1 {
//         assert_eq!(k, 0);
//         return x.first().unwrap();
//     }

//     let pivot = get_pivot(x);

//     let mut lessers = Vec::new();
//     let mut greaters = Vec::new();
//     let mut equal_count = 0;

//     for &x in x {
//         match x.cmp(pivot) {
//             std::cmp::Ordering::Less => lessers.push(x),
//             std::cmp::Ordering::Equal => equal_count += 1,
//             std::cmp::Ordering::Greater => greaters.push(x),
//         }
//     }

//     if k < lessers.len() {
//         quick_select_ref(&lessers, k, get_pivot)
//     } else if k < lessers.len() + equal_count {
//         pivot
//     } else {
//         quick_select_ref(&greaters, k - lessers.len() - equal_count, get_pivot)
//     }
// }

// fn quick_select_val<T: Ord + Copy>(x: &[T], k: usize, get_pivot: impl Fn(&[T]) -> T) -> T {
//     if x.len() == 1 {
//         assert_eq!(k, 0);
//         return x.first().copied().unwrap();
//     }

//     let pivot = get_pivot(x);

//     let mut lessers = Vec::new();
//     let mut greaters = Vec::new();
//     let mut equal_count = 0;

//     for &x in x {
//         match x.cmp(&pivot) {
//             std::cmp::Ordering::Less => lessers.push(x),
//             std::cmp::Ordering::Equal => equal_count += 1,
//             std::cmp::Ordering::Greater => greaters.push(x),
//         }
//     }

//     if k < lessers.len() {
//         quick_select_val(&lessers, k, get_pivot)
//     } else if k < lessers.len() + equal_count {
//         pivot
//     } else {
//         quick_select_val(&greaters, k - lessers.len() - equal_count, get_pivot)
//     }
// }

// fn median<T: Ord + Copy>(x: &[T]) -> T {
//     quick_select_val(x, x.len() / 2, |x| x[x.len() / 2])
// }

// fn find_peaks_indices(x: &[f32]) -> Vec<usize> {
//     let mut peaks = Vec::new();

//     let max = x.len() - 1;
//     for i in 1..max {
//         if x[i - 1] < x[i] && x[i] > x[i + 1] {
//             peaks.push(i);
//         }
//     }

//     peaks
// }
fn find_peaks_indices<S, T>(x: &ArrayBase<S, Ix1>) -> Vec<usize>
where
    S: ndarray::Data<Elem = T>,
    T: PartialOrd,
{
    let mut peaks = vec![0];

    let max = x.len() - 1;
    for i in 1..max {
        if x[i - 1] < x[i] && x[i] > x[i + 1] {
            peaks.push(i);
        }
    }

    peaks.push(x.len());
    peaks
}

pub fn diff<T: Sub<Output = T> + Clone>(x: &[T]) -> Vec<T> {
    x.windows(2).map(|w| w[1].clone() - w[0].clone()).collect()
}

pub fn diff_nd<S, D, T>(x: &ArrayBase<S, D>, axis: Axis) -> Array<T, D>
where
    S: ndarray::Data<Elem = T>,
    T: Sub<Output = T> + Clone,
    D: Dimension,
{
    let a = x.slice_each_axis(|d| {
        if d.axis == axis {
            Slice::from(1..)
        } else {
            Slice::from(..)
        }
    });
    let b = x.slice_each_axis(|d| {
        if d.axis == axis {
            Slice::from(..-1)
        } else {
            Slice::from(..)
        }
    });

    a.to_owned() - b
}

use utils::get_median;

use crate::utils::Median;

// fn median<S, T>(x: &ArrayBase<S, Ix1>) -> T
// where
//     S: ndarray::Data<Elem = T>,
//     T: Ord + Clone,
// {
//     let mut x = x.to_vec();
//     x.sort();
//     x.swap_remove(x.len() / 2)
// }

fn median(x: &mut [usize]) -> usize {
    x.sort();
    if x.len() % 2 == 0 {
        (x[x.len() / 2] + x[x.len() / 2 - 1]) / 2
    } else {
        x[x.len() / 2]
    }
}

fn main() -> Result<()> {
    let a = ImageReader::open("sandbox/690292005999224951.png")?.decode()?;

    if let Some(im) = a.as_rgb8() {
        println!("Image is RGB8");
        let data = im.ref_ndarray3(); // 3xHxW
        println!("{:?}", data.dim());
        let width = data.dim().2;
        let height = data.dim().1;

        let data = data.map(|&x| unsafe { NotNan::new_unchecked(x as f32) }); // Safety: all range of u8 can be converted to f32

        let hsize = {
            // let a = data.slice(s!(.., .., 1..)); // Everything except the first column
            // let b = data.slice(s!(.., .., ..-1)); // Everything except the last column
            // let hdiff = (a.to_owned() - b)
            let hdiff = diff_nd(&data, Axis(2))
                .mapv_into(|x| x * x)
                .sum_axis(Axis(0)) // collapse channels
                .mapv_into(|x| unsafe { NotNan::new_unchecked(x.into_inner().sqrt()) }); // Safety: sum of 3 squares (positive) cannot be negative
            let hsum = hdiff.sum_axis(Axis(0));

            let hpeaks = find_peaks_indices(&hsum);

            // let a = hpeaks.slice(s![1..]);
            // let b = hpeaks.slice(s![..-1]);
            // let hspacing = a.to_owned() - b;
            let mut hspacing = NonEmptyVec::new(diff(&hpeaks)).unwrap();

            get_median(&mut hspacing).reduce(|a, b| {
                if (a + b) % 2 == 0 {
                    (a + b) / 2
                } else if width % b == 0 {
                    b
                } else {
                    a
                }
            })

            // median(&mut hspacing)
        };

        let vsize = {
            // let a = data.slice(s!(.., 1.., ..)); // Everything except the first row
            // let b = data.slice(s!(.., ..-1, ..)); // Everything except the last row
            // let vdiff = (a.to_owned() - b)
            let vdiff = diff_nd(&data, Axis(1))
                .mapv_into(|x| x * x)
                .sum_axis(Axis(0)) // collapse channels
                .mapv_into(|x| unsafe { NotNan::new_unchecked(x.into_inner().sqrt()) }); // Safety: sum of 3 squares (positive) cannot be negative
            let vsum = vdiff.sum_axis(Axis(1));

            let vpeaks = find_peaks_indices(&vsum);

            // let a = vpeaks.slice(s![1..]);
            // let b = vpeaks.slice(s![..-1]);
            // let vspacing = a.to_owned() - b;
            let mut vspacing = NonEmptyVec::new(diff(&vpeaks)).unwrap();

            get_median(&mut vspacing).reduce(|a, b| {
                if (a + b) % 2 == 0 {
                    (a + b) / 2
                } else if width % b == 0 {
                    b
                } else {
                    a
                }
            })

            // median(&mut vspacing)
        };

        let pixel_size = gcd(hsize, vsize);

        println!("pixel size: {}", pixel_size);
        assert!(width % pixel_size == 0);
        assert!(height % pixel_size == 0);

        let new_width = width / pixel_size;
        let new_height = height / pixel_size;

        thumbnail(im, new_width as u32, new_height as u32).save("sandbox/direct.png")?;
    }

    Ok(())
}
