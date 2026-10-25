//! A scenario's frames (a step's `screenshots_every`, `tick-NNNN.png`) as an animated GIF that
//! plays them in game time, each frame shown until the next one's tick: one run's frames alone,
//! or two runs' side by side with a heatmap of where they differ.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::Duration;

use image::codecs::gif::{GifEncoder, Repeat};
use image::imageops::{self, FilterType};
use image::{Delay, Frame, RgbaImage};

use crate::screenshot;

/// A tick's length in milliseconds, at the game's 20 ticks a second.
const TICK_MILLIS: u64 = 50;

/// The frames' ticks and paths in `dir/screenshots/`, in tick order.
fn frames(dir: &Path) -> Result<Vec<(u32, PathBuf)>, String> {
    let screenshots = dir.join("screenshots");
    let entries = fs::read_dir(&screenshots)
        .map_err(|error| format!("{}: {error}", screenshots.display()))?;
    let mut frames: Vec<(u32, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let tick = name
                .strip_prefix("tick-")?
                .strip_suffix(".png")?
                .parse()
                .ok()?;
            Some((tick, entry.path()))
        })
        .collect();
    frames.sort();
    if frames.is_empty() {
        return Err(format!(
            "no tick-NNNN.png frames in {}",
            screenshots.display()
        ));
    }
    Ok(frames)
}

fn open(path: &Path) -> Result<RgbaImage, String> {
    image::open(path)
        .map(|image| image.to_rgba8())
        .map_err(|error| format!("{}: {error}", path.display()))
}

/// Images side by side, left to right.
fn side_by_side(images: &[RgbaImage]) -> RgbaImage {
    let width = images.iter().map(RgbaImage::width).sum();
    let height = images.iter().map(RgbaImage::height).max().unwrap_or(0);
    let mut out = RgbaImage::new(width, height);
    let mut x = 0;
    for image in images {
        imageops::replace(&mut out, image, i64::from(x), 0);
        x += image.width();
    }
    out
}

/// Writes `a`'s frames, or `a`'s, `b`'s and their heatmap side by side when `b` is given, to
/// `out`, each `1 / scale` the size. Every frame of `a` needs one of the same tick in `b`.
pub fn write(a: &Path, b: Option<&Path>, out: &Path, scale: u32) -> Result<usize, String> {
    let frames_a = frames(a)?;
    let file = File::create(out).map_err(|error| format!("{}: {error}", out.display()))?;
    let mut encoder = GifEncoder::new_with_speed(file, 10);
    encoder
        .set_repeat(Repeat::Infinite)
        .map_err(|error| error.to_string())?;
    for (index, (tick, path)) in frames_a.iter().enumerate() {
        let image_a = open(path)?;
        let image = match b {
            None => image_a,
            Some(b) => {
                let path_b = b
                    .join("screenshots")
                    .join(path.file_name().expect("a frame"));
                let image_b = open(&path_b)?;
                let heatmap = screenshot::heatmap(&image_a, &image_b, 0);
                side_by_side(&[image_a, image_b, heatmap])
            }
        };
        let image = match scale {
            1 => image,
            _ => imageops::resize(
                &image,
                image.width() / scale,
                image.height() / scale,
                FilterType::Triangle,
            ),
        };
        // Until the next frame's tick; the last as long as the one before it.
        let ticks = match (frames_a.get(index + 1), index.checked_sub(1)) {
            (Some((next, _)), _) => next - tick,
            (None, Some(previous)) => tick - frames_a[previous].0,
            (None, None) => 1,
        };
        let delay =
            Delay::from_saturating_duration(Duration::from_millis(u64::from(ticks) * TICK_MILLIS));
        encoder
            .encode_frame(Frame::from_parts(image, 0, 0, delay))
            .map_err(|error| format!("{}: {error}", out.display()))?;
    }
    Ok(frames_a.len())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use image::codecs::gif::GifDecoder;
    use image::{AnimationDecoder, Rgba, RgbaImage};

    /// A run's directory of frames, each a single colour, at the ticks given.
    fn run(dir: &Path, frames: &[(u32, u8)]) {
        std::fs::create_dir_all(dir.join("screenshots")).unwrap();
        for &(tick, shade) in frames {
            RgbaImage::from_pixel(8, 4, Rgba([shade, shade, shade, 255]))
                .save(dir.join(format!("screenshots/tick-{tick:04}.png")))
                .unwrap();
        }
        // Not a frame of the video: a screenshot step's.
        RgbaImage::new(8, 4)
            .save(dir.join("screenshots/spawn.png"))
            .unwrap();
    }

    #[test]
    fn plays_frames_in_game_time_side_by_side() {
        let dir = std::env::temp_dir().join(format!("oracle-diff-video-{}", std::process::id()));
        let (a, b) = (dir.join("a"), dir.join("b"));
        run(&a, &[(3, 0), (7, 128), (9, 255)]);
        run(&b, &[(3, 0), (7, 128), (9, 200)]);

        let out = dir.join("a.gif");
        assert_eq!(super::write(&a, None, &out, 1).unwrap(), 3);
        let frames = GifDecoder::new(std::io::BufReader::new(std::fs::File::open(&out).unwrap()))
            .unwrap()
            .into_frames()
            .collect_frames()
            .unwrap();
        // Until the next frame's tick, the last as long as the one before.
        let delays: Vec<u32> = frames
            .iter()
            .map(|frame| {
                let (numerator, denominator) = frame.delay().numer_denom_ms();
                numerator / denominator
            })
            .collect();
        assert_eq!(delays, [200, 100, 100]);
        assert_eq!(frames[0].buffer().dimensions(), (8, 4));

        // A, B and the heatmap, which marks the last frame's pixels.
        let out = dir.join("ab.gif");
        assert_eq!(super::write(&a, Some(&b), &out, 1).unwrap(), 3);
        let frames = GifDecoder::new(std::io::BufReader::new(std::fs::File::open(&out).unwrap()))
            .unwrap()
            .into_frames()
            .collect_frames()
            .unwrap();
        assert_eq!(frames[2].buffer().dimensions(), (24, 4));
        // The same pixels in grey, the differing ones in red.
        let [red, green, _, _] = frames[1].buffer().get_pixel(20, 2).0;
        assert_eq!(red, green);
        let [red, green, _, _] = frames[2].buffer().get_pixel(20, 2).0;
        assert!(red > 64 && green == 0);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
