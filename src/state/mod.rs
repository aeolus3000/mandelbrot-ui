use std::borrow::Borrow;
use tetra::{Context, State};
use num::Complex;
use rayon::prelude::*;
use tetra::math::Vec2;
use tetra::graphics::{self, DrawParams, ImageData, Texture};

pub struct GameState {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
    mandelbrot_texture: Texture,
}

impl GameState {
    pub fn new(ctx: &mut Context, window_width: usize, window_height: usize) -> tetra::Result<GameState> {
        let pixels = vec![0; window_width * window_height * 4];
        let id = ImageData::from_rgba8(window_width as i32, window_height as i32, pixels.as_slice())?;
        let mandelbrot_texture = Texture::from_image_data(ctx, &id)?;
        Ok(GameState {pixels, width: window_width, height: window_height, mandelbrot_texture})
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        // -1.20,0.35 -1,0.20
        // Scope of slicing up `pixels` into horizontal bands.
        {
            let upper_left = Complex { re: -1.20, im: 0.35 };
            let lower_right = Complex { re: -1.0, im: 0.2 };
            let bounds = (self.width, self.height);
            let bands: Vec<(usize, &mut [u8])> = self.pixels
                .chunks_mut(bounds.0 * 4)
                .enumerate()
                .collect();

            bands.into_par_iter()
                .for_each(|(i, band)| {
                    let top = i;
                    let band_bounds = (bounds.0, 1);
                    let band_upper_left = pixel_to_point(bounds, (0, top),
                                                         upper_left, lower_right);
                    let band_lower_right = pixel_to_point(bounds, (bounds.0, top + 1),
                                                          upper_left, lower_right);
                    render(band, band_bounds, band_upper_left, band_lower_right);
                });
        }

        let mandelbrot_position = Vec2::new(
            0.0,
            0.0,
        );
        println!("Print mandelbrot");
        self.mandelbrot_texture.replace_data(ctx, self.pixels.as_slice());
        self.mandelbrot_texture.draw(ctx, mandelbrot_position);
        println!("Printed mandelbrot");
        Ok(())
    }
}


/// Try to determine if `c` is in the Mandelbrot set, using at most `limit`
/// iterations to decide.
///
/// If `c` is not a member, return `Some(i)`, where `i` is the number of
/// iterations it took for `c` to leave the circle of radius two centered on the
/// origin. If `c` seems to be a member (more precisely, if we reached the
/// iteration limit without being able to prove that `c` is not a member),
/// return `None`.
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }

    None
}

/// Given the row and column of a pixel in the output image, return the
/// corresponding point on the complex plane.
///
/// `bounds` is a pair giving the width and height of the image in pixels.
/// `pixel` is a (column, row) pair indicating a particular pixel in that image.
/// The `upper_left` and `lower_right` parameters are points on the complex
/// plane designating the area our image covers.
fn pixel_to_point(bounds: (usize, usize),
                  pixel: (usize, usize),
                  upper_left: Complex<f64>,
                  lower_right: Complex<f64>)
                  -> Complex<f64>
{
    let (width, height) = (lower_right.re - upper_left.re,
                           upper_left.im - lower_right.im);
    Complex {
        re: upper_left.re + pixel.0 as f64 * width  / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
        // Why subtraction here? pixel.1 increases as we go down,
        // but the imaginary component increases as we go up.
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100, 200), (25, 175),
                              Complex { re: -1.0, im:  1.0 },
                              Complex { re:  1.0, im: -1.0 }),
               Complex { re: -0.5, im: -0.75 });
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
///
/// The `bounds` argument gives the width and height of the buffer `pixels`,
/// which holds one grayscale pixel per byte. The `upper_left` and `lower_right`
/// arguments specify points on the complex plane corresponding to the upper-
/// left and lower-right corners of the pixel buffer.
fn render(pixels: &mut [u8],
          bounds: (usize, usize),
          upper_left: Complex<f64>,
          lower_right: Complex<f64>)
{
    assert!(pixels.len() == bounds.0 * bounds.1 * 4);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row),
                                       upper_left, lower_right);

            for color in 0..4 {
                pixels[row * bounds.0 + (column * 4) + color] =
                    match escape_time(point, 255) {
                        None => 0,
                        Some(count) => 255 - count as u8
                    };
            }

        }
    }
}
