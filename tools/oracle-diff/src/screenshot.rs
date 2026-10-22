use std::fmt;

use image::{Rgba, RgbaImage};

pub struct ImageDiff {
    pub width: u32,
    pub height: u32,
    pub threshold: u8,
    pub differing: u64,
    pub max_delta: u8,
    /// Smallest rectangle holding every differing pixel: x0, y0, x1, y1, inclusive.
    pub bounds: Option<[u32; 4]>,
}

impl ImageDiff {
    pub fn is_same(&self) -> bool {
        self.differing == 0
    }
}

// The largest difference across the four channels.
fn delta(a: &Rgba<u8>, b: &Rgba<u8>) -> u8 {
    a.0.iter()
        .zip(b.0)
        .map(|(x, y)| x.abs_diff(y))
        .max()
        .unwrap_or(0)
}

/// Counts pixels whose largest channel difference is above `threshold` (0 means exact).
pub fn compare(a: &RgbaImage, b: &RgbaImage, threshold: u8) -> Result<ImageDiff, String> {
    if a.dimensions() != b.dimensions() {
        return Err(format!(
            "sizes differ: {}x{} vs {}x{}",
            a.width(),
            a.height(),
            b.width(),
            b.height()
        ));
    }

    let mut diff = ImageDiff {
        width: a.width(),
        height: a.height(),
        threshold,
        differing: 0,
        max_delta: 0,
        bounds: None,
    };
    for ((x, y, pixel_a), pixel_b) in a.enumerate_pixels().zip(b.pixels()) {
        let delta = delta(pixel_a, pixel_b);
        diff.max_delta = diff.max_delta.max(delta);
        if delta <= threshold {
            continue;
        }
        diff.differing += 1;
        diff.bounds = Some(match diff.bounds {
            None => [x, y, x, y],
            Some([x0, y0, x1, y1]) => [x0.min(x), y0.min(y), x1.max(x), y1.max(y)],
        });
    }
    Ok(diff)
}

/// A dimmed grey copy of `a` with the differing pixels in red, brighter the bigger the difference.
pub fn heatmap(a: &RgbaImage, b: &RgbaImage, threshold: u8) -> RgbaImage {
    RgbaImage::from_fn(a.width(), a.height(), |x, y| {
        let (pixel_a, pixel_b) = (a.get_pixel(x, y), b.get_pixel(x, y));
        let delta = delta(pixel_a, pixel_b);
        if delta > threshold {
            return Rgba([64 + (u16::from(delta) * 191 / 255) as u8, 0, 0, 255]);
        }
        let [r, g, b, _] = pixel_a.0;
        let grey = ((u16::from(r) + u16::from(g) + u16::from(b)) / 12) as u8;
        Rgba([grey, grey, grey, 255])
    })
}

impl fmt::Display for ImageDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = u64::from(self.width) * u64::from(self.height);
        write!(f, "{}x{}, ", self.width, self.height)?;
        let Some([x0, y0, x1, y1]) = self.bounds else {
            return write!(
                f,
                "identical (max channel delta {} within threshold {})",
                self.max_delta, self.threshold
            );
        };
        write!(
            f,
            "{} of {total} pixels differ by more than {} ({:.3}%), max channel delta {}, within x {x0}..={x1} y {y0}..={y1}",
            self.differing,
            self.threshold,
            self.differing as f64 * 100.0 / total as f64,
            self.max_delta,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(width: u32, height: u32, colour: [u8; 4]) -> RgbaImage {
        RgbaImage::from_pixel(width, height, Rgba(colour))
    }

    #[test]
    fn identical_images_match() {
        let image = solid(4, 3, [10, 20, 30, 255]);
        assert!(compare(&image, &image, 0).unwrap().is_same());
    }

    #[test]
    fn counts_pixels_over_the_threshold_and_their_bounds() {
        let a = solid(8, 8, [100, 100, 100, 255]);
        let mut b = a.clone();
        b.put_pixel(2, 3, Rgba([103, 100, 100, 255]));
        b.put_pixel(5, 6, Rgba([100, 140, 100, 255]));

        let exact = compare(&a, &b, 0).unwrap();
        assert_eq!(
            (exact.differing, exact.max_delta, exact.bounds),
            (2, 40, Some([2, 3, 5, 6]))
        );

        let tolerant = compare(&a, &b, 5).unwrap();
        assert_eq!(
            (tolerant.differing, tolerant.bounds),
            (1, Some([5, 6, 5, 6]))
        );
    }

    #[test]
    fn different_sizes_are_an_error() {
        assert!(compare(&solid(2, 2, [0; 4]), &solid(3, 2, [0; 4]), 0).is_err());
    }

    #[test]
    fn heatmap_marks_differences_red() {
        let a = solid(2, 1, [90, 90, 90, 255]);
        let mut b = a.clone();
        b.put_pixel(1, 0, Rgba([0, 90, 90, 255]));
        let map = heatmap(&a, &b, 0);
        assert_eq!(map.get_pixel(0, 0).0, [22, 22, 22, 255]);
        assert!(map.get_pixel(1, 0).0[0] > 64 && map.get_pixel(1, 0).0[1] == 0);
    }
}
