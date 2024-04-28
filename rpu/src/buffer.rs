//use crate::prelude::*;
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

/// A color buffer holding an array of f64 pixels.
#[derive(PartialEq, Debug, Clone)]
pub struct ColorBuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<f64>,
    pub frames: usize,
}

impl ColorBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0.0; width * height * 4],
            frames: 0,
        }
    }

    #[inline(always)]
    pub fn at(&self, x: usize, y: usize) -> [f64; 4] {
        let i = y * self.width * 4 + x * 4;
        [
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        ]
    }

    pub fn set(&mut self, x: usize, y: usize, color: [f64; 4]) {
        let i = y * self.width * 4 + x * 4;
        self.pixels[i..i + 4].copy_from_slice(&color);
    }

    pub fn set_pixels(&mut self, x: usize, y: usize, width: usize, height: usize, pixels: &[f64]) {
        for local_y in 0..height {
            for local_x in 0..width {
                let global_x = x + local_x;
                let global_y = y + local_y;

                if global_x >= self.width || global_y >= self.height {
                    continue;
                }

                let index = (global_y * self.width + global_x) * 4;
                let local_index = (local_y * width + local_x) * 4;
                self.pixels[index..index + 4]
                    .copy_from_slice(&pixels[local_index..local_index + 4]);
            }
        }
    }

    /// Convert the frame to an u8 vec, applying gamma correction
    pub fn to_u8_vec_gamma(&self) -> Vec<u8> {
        let source = &self.pixels[..];
        let mut out: Vec<u8> = vec![0; self.width * self.height * 4];
        let gamma_correction = 0.4545;

        for y in 0..self.height {
            for x in 0..self.width {
                let d = x * 4 + y * self.width * 4;
                let c = [
                    (source[d].powf(gamma_correction) * 255.0) as u8,
                    (source[d + 1].powf(gamma_correction) * 255.0) as u8,
                    (source[d + 2].powf(gamma_correction) * 255.0) as u8,
                    (source[d + 3] * 255.0) as u8,
                ];
                out[d..d + 4].copy_from_slice(&c);
            }
        }

        out
    }

    /// Convert the frame to an u8 vec, applying gamma correction
    pub fn to_u8_vec(&self) -> Vec<u8> {
        let source = &self.pixels[..];
        let mut out: Vec<u8> = vec![0; self.width * self.height * 4];

        for y in 0..self.height {
            for x in 0..self.width {
                let d = x * 4 + y * self.width * 4;
                let c = [
                    (source[d] * 255.0) as u8,
                    (source[d + 1] * 255.0) as u8,
                    (source[d + 2] * 255.0) as u8,
                    (source[d + 3] * 255.0) as u8,
                ];
                out[d..d + 4].copy_from_slice(&c);
            }
        }

        out
    }

    /// Convert the pixel buffer to an Vec<u8> and converts the colors from linear into gamma space in a parallel fashion.
    pub fn convert_to_u8_at(&self, frame: &mut [u8], at: (usize, usize, usize, usize)) {
        let (start_x, start_y, width, _height) = at;

        frame
            .par_chunks_exact_mut(width * 4)
            .enumerate()
            .for_each(|(j, line)| {
                for (i, pixel) in line.chunks_exact_mut(4).enumerate() {
                    let x = start_x + i % width;
                    let y = start_y + j;

                    if x < self.width && y < self.height {
                        let o = y * self.width * 4 + x * 4;
                        let c = [
                            (self.pixels[o].powf(0.4545) * 255.0) as u8,
                            (self.pixels[o + 1].powf(0.4545) * 255.0) as u8,
                            (self.pixels[o + 2].powf(0.4545) * 255.0) as u8,
                            (self.pixels[o + 3] * 255.0) as u8,
                        ];
                        pixel.copy_from_slice(&c);
                    }
                }
            });
    }

    pub fn save(&self, path: std::path::PathBuf) {
        let mut image = image::ImageBuffer::new(self.width as u32, self.height as u32);

        for y in 0..self.height {
            for x in 0..self.width {
                let i = y * self.width * 4 + x * 4;
                let c = image::Rgb([
                    (self.pixels[i] * 255.0) as u8,
                    (self.pixels[i + 1] * 255.0) as u8,
                    (self.pixels[i + 2] * 255.0) as u8,
                ]);
                image.put_pixel(x as u32, y as u32, c);
            }
        }

        image.save(path).unwrap();
    }
}
