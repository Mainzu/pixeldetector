use std::collections::HashMap;
use std::io::Cursor;
use std::ops::Sub;

use anyhow::{Ok, Result};
use image::flat::SampleLayout;
use image::imageops::thumbnail;
use image::{io::Reader as ImageReader, Pixel};
use image::{EncodableLayout, FlatSamples, GenericImageView, ImageFormat, Rgb};
use ndarray::{self, Data};
use ndarray::{prelude::*, Slice};
use nshare::{RefNdarray3, ToImageLuma};
use num_integer::{gcd, Integer};
use ordered_float::NotNan;

fn print_stats(image: &image::DynamicImage) {
    println!(
        "{w}x{h} {ct:?} {s} bytes",
        w = image.width(),
        h = image.height(),
        ct = image.color(),
        s = image.as_bytes().len()
    );
}

#[inline]
fn u8_to_f32(x: u8) -> NotNan<f32> {
    unsafe { NotNan::new_unchecked(x as f32) }
}

#[inline]
fn square(x: f32) -> f32 {
    x * x
}

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
    let mut peaks = Vec::new();

    let max = x.len() - 1;
    for i in 1..max {
        if x[i - 1] < x[i] && x[i] > x[i + 1] {
            peaks.push(i);
        }
    }

    peaks
}

pub fn diff<S, D, T>(x: &ArrayBase<S, D>, axis: Axis) -> Array<T, D>
where
    S: Data<Elem = T>,
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

fn median<S, T>(x: &ArrayBase<S, Ix1>) -> T
where
    S: ndarray::Data<Elem = T>,
    T: Ord + Clone,
{
    let mut x = x.to_vec();
    x.sort();
    x.swap_remove(x.len() / 2)
}

fn main() -> Result<()> {
    let a = ImageReader::open("sandbox/690292005999224951.png")?.decode()?;
    print_stats(&a);

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
            let hdiff = diff(&data, Axis(2))
                .mapv_into(|x| x * x)
                .sum_axis(Axis(0)) // collapse channels
                .mapv_into(|x| unsafe { NotNan::new_unchecked(x.into_inner().sqrt()) }); // Safety: sum of 3 squares (positive) cannot be negative
            let hsum = hdiff.sum_axis(Axis(0));

            let hpeaks = Array::from(find_peaks_indices(&hsum));

            // let a = hpeaks.slice(s![1..]);
            // let b = hpeaks.slice(s![..-1]);
            // let hspacing = a.to_owned() - b;
            let hspacing = diff(&hpeaks, Axis(0));

            median(&hspacing)
        };

        let vsize = {
            // let a = data.slice(s!(.., 1.., ..)); // Everything except the first row
            // let b = data.slice(s!(.., ..-1, ..)); // Everything except the last row
            // let vdiff = (a.to_owned() - b)
            let vdiff = diff(&data, Axis(1))
                .mapv_into(|x| x * x)
                .sum_axis(Axis(0)) // collapse channels
                .mapv_into(|x| unsafe { NotNan::new_unchecked(x.into_inner().sqrt()) }); // Safety: sum of 3 squares (positive) cannot be negative
            let vsum = vdiff.sum_axis(Axis(1));

            // if hsum.is_standard_layout() {
            //     println!("hsum is standard layout");
            // }
            // if vsum.is_standard_layout() {
            //     println!("vsum is standard layout");
            // }

            let vpeaks = Array::from(find_peaks_indices(&vsum));

            // let a = vpeaks.slice(s![1..]);
            // let b = vpeaks.slice(s![..-1]);
            // let vspacing = a.to_owned() - b;
            let vspacing = diff(&vpeaks, Axis(0));

            median(&vspacing)
        };

        let pixel_size = gcd(hsize, vsize);

        println!("pixel size: {}", pixel_size);
        assert!(width % pixel_size == 0);
        assert!(height % pixel_size == 0);

        let new_width = width / pixel_size;
        let new_height = height / pixel_size;

        thumbnail(im, new_width as u32, new_height as u32).save("sandbox/direct.png")?;

        // let data = im.ref_ndarray3().map_axis(Axis(0), |channels| {
        //     image::Rgb::<u8>([channels[0], channels[1], channels[2]])
        // });
        // let mut downsized = Array::zeros((new_width, new_height, 3));
        // for x in 0..new_width {
        //     for y in 0..new_height {
        //         // let tile = im.view(x * pixel_size, y * pixel_size, pixel_size, pixel_size);
        //         let tile = data.slice(s![
        //             y * pixel_size..(y + 1) * pixel_size,
        //             x * pixel_size..(x + 1) * pixel_size,
        //         ]);
        //         // Get most common color
        //         let freq = tile.fold(HashMap::new(), |mut acc, pixel| {
        //             *acc.entry(*pixel).or_insert(0) += 1;
        //             acc
        //         });

        //         let (color, _) = freq.into_iter().max_by(|(_, a), (_, b)| a.cmp(b)).unwrap();
        //         // println!("{} {} rgb({}, {}, {})", x, y, color[0], color[1], color[2]);
        //         downsized[[x, y, 0]] = color[0];
        //         downsized[[x, y, 1]] = color[1];
        //         downsized[[x, y, 2]] = color[2];
        //     }
        // }

        // let buffer;

        // let bytes = if let Some(slice) = downsized.as_slice() {
        //     slice
        // } else {
        //     buffer = downsized.into_raw_vec();
        //     buffer.as_bytes()
        // };

        // let b = FlatSamples {
        //     samples: bytes,
        //     layout: SampleLayout::column_major_packed(3 as u8, new_width as u32, new_height as u32),
        //     color_hint: Some(image::ColorType::Rgb8),
        // };

        // let view = b.as_view::<Rgb<u8>>()?;
        // thumbnail(&view, new_width as u32, new_height as u32).save("sandbox/thumbnail.png")?;
    }

    Ok(())
}
