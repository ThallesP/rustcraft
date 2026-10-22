//! com/mojang/blaze3d/platform/TextureUtil.java
//!
//! Only what mipmaps need so far: `solidify` and `fillEmptyAreasWithDarkColor`, which give a
//! sprite's transparent pixels a color before it's mipmapped. `readResource` is reading the bytes,
//! and `writeAsPNG` (an atlas's dump) comes with the debug dumps.

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::net::minecraft::util::argb::ARGB;
use std::collections::VecDeque;

pub struct TextureUtil;

const DIRECTIONS: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];

impl TextureUtil {
    /// Gives each transparent pixel the color of the nearest opaque one (breadth first, in
    /// `DIRECTIONS`' order), keeping it transparent.
    pub fn solidify(image: &mut NativeImage) {
        let width = image.get_width();
        let height = image.get_height();
        let mut nearest_color = vec![0; (width * height) as usize];
        let mut distances = vec![i32::MAX; (width * height) as usize];
        let mut queue = VecDeque::new();

        for x in 0..width {
            for y in 0..height {
                let color = image.get_pixel(x, y);
                if ARGB::alpha(color) != 0 {
                    let packed_coordinates = TextureUtil::pack(x, y, width);
                    distances[packed_coordinates as usize] = 0;
                    nearest_color[packed_coordinates as usize] = color;
                    queue.push_back(packed_coordinates);
                }
            }
        }

        while let Some(packed_coordinates) = queue.pop_front() {
            let x = TextureUtil::x(packed_coordinates, width);
            let y = TextureUtil::y(packed_coordinates, width);
            for direction in DIRECTIONS {
                let neighbor_x = x + direction[0];
                let neighbor_y = y + direction[1];
                let packed_neighbor_coordinates = TextureUtil::pack(neighbor_x, neighbor_y, width);
                if neighbor_x >= 0
                    && neighbor_y >= 0
                    && neighbor_x < width
                    && neighbor_y < height
                    && distances[packed_neighbor_coordinates as usize]
                        > distances[packed_coordinates as usize] + 1
                {
                    distances[packed_neighbor_coordinates as usize] =
                        distances[packed_coordinates as usize] + 1;
                    nearest_color[packed_neighbor_coordinates as usize] =
                        nearest_color[packed_coordinates as usize];
                    queue.push_back(packed_neighbor_coordinates);
                }
            }
        }

        for x in 0..width {
            for y in 0..height {
                let color = image.get_pixel(x, y);
                if ARGB::alpha(color) == 0 {
                    let nearest = nearest_color[TextureUtil::pack(x, y, width) as usize];
                    image.set_pixel(x, y, ARGB::color_alpha(0, nearest));
                }
            }
        }
    }

    /// Gives each transparent pixel three quarters of the darkest opaque pixel's color.
    pub fn fill_empty_areas_with_dark_color(image: &mut NativeImage) {
        let width = image.get_width();
        let height = image.get_height();
        let mut darkest_color = -1;
        let mut min_brightness = i32::MAX;

        for x in 0..width {
            for y in 0..height {
                let color = image.get_pixel(x, y);
                let alpha = ARGB::alpha(color);
                if alpha != 0 {
                    let brightness = ARGB::red(color) + ARGB::green(color) + ARGB::blue(color);
                    if brightness < min_brightness {
                        min_brightness = brightness;
                        darkest_color = color;
                    }
                }
            }
        }

        let dark_red = 3 * ARGB::red(darkest_color) / 4;
        let dark_green = 3 * ARGB::green(darkest_color) / 4;
        let dark_blue = 3 * ARGB::blue(darkest_color) / 4;
        let darkened_color = ARGB::color(0, dark_red, dark_green, dark_blue);

        for x in 0..width {
            for y in 0..height {
                let color = image.get_pixel(x, y);
                if ARGB::alpha(color) == 0 {
                    image.set_pixel(x, y, darkened_color);
                }
            }
        }
    }

    fn pack(x: i32, y: i32, width: i32) -> i32 {
        x + y * width
    }

    fn x(packed: i32, width: i32) -> i32 {
        packed % width
    }

    fn y(packed: i32, width: i32) -> i32 {
        packed / width
    }
}
