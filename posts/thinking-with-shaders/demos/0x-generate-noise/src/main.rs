fn main() -> Result<(), Box<dyn Error>> {
    let noise = Noise::from((
        LayeredNoise::new(
            Normed::<f32>::default(),
            Persistence::default(),
            FractalLayers {
                layer: Octave(MixCellGradients::<_, _, _, false> {
                    cells: OrthoGrid(16),
                    gradients: QuickGradients,
                    curve: Smoothstep,
                }),
                lacunarity: 2.0,
                // yes i know 1 doesn't do anything
                // but it makes it easier to customise later
                amount: 1,
            },
        ),
        SNormToUNorm,
    ));
    let size = 128;
    let mut imgbuf = image::ImageBuffer::new(size, size);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let (x, y) = (x as f32 / 8.0, y as f32 / 8.0);
        let noise: f32 = noise.sample(Vec2::new(x, y));
        dbg!(noise);
        *pixel = Rgb([(noise * 255.0) as u8; 3]);
    }
    fs::create_dir_all("../assets")?;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("images/noise.webp")?;
    imgbuf.write_to(&mut file, ImageFormat::WebP)?;

    Ok(())
}

use std::{
    error::Error,
    fs::{self, OpenOptions},
};

use bevy_math::Vec2;
use image::{ImageFormat, Rgb};
use noiz::prelude::*;
