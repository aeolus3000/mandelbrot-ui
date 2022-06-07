#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

use tetra::ContextBuilder;
use mandelbrot::state::GameState;
use num::Complex;
use rayon::prelude::*;

use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;

/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize))
    -> Result<(), std::io::Error>
{
    let output = File::create(filename)?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(&pixels,
                   bounds.0 as u32, bounds.1 as u32,
                   ColorType::Gray(8))?;

    Ok(())
}

use std::env;

fn main() -> tetra::Result {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} PIXELS",
                  args[0]);
        eprintln!("Example: {} 1000x750",
                  args[0]);
        std::process::exit(1);
    }

    let bounds: (usize, usize) = parse_pair(&args[1], 'x')
        .expect("error parsing image dimensions");

    let ran = ContextBuilder::new("Mandelbrot", bounds.0 as i32, bounds.1 as i32)
        .quit_on_escape(true)
        .build()?
        .run(|ctx| { GameState::new(ctx, bounds.0 as usize, bounds.1 as usize) });

    // debugging / error / or whatever
    match ran {
        Ok(_) => {
            println!("Hurra");
        }
        Err(_) => {
            println!("Flark frigging dragons frigged it up again");
        }
    }

    ran

    // write_image(&args[1], &pixels, bounds)
    //     .expect("error writing PNG file");
}



use std::str::FromStr;
use tetra::graphics::Texture;

/// Parse the string `s` as a coordinate pair, like `"400x600"` or `"1.0,0.5"`.
///
/// Specifically, `s` should have the form <left><sep><right>, where <sep> is
/// the character given by the `separator` argument, and <left> and <right> are both
/// strings that can be parsed by `T::from_str`.
///
/// If `s` has the proper form, return `Some<(x, y)>`. If it doesn't parse
/// correctly, return `None`.
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            let a = (T::from_str(&s[..index]), T::from_str(&s[index + 1..]));
            match a {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("",        ','), None);
    assert_eq!(parse_pair::<i32>("10,",     ','), None);
    assert_eq!(parse_pair::<i32>(",10",     ','), None);
    assert_eq!(parse_pair::<i32>("10,20",   ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x",    'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

/// Parse a pair of floating-point numbers separated by a comma as a complex
/// number.
fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None
    }
}

#[test]
fn test_parse_complex() {
    assert_eq!(parse_complex("1.25,-0.0625"),
               Some(Complex { re: 1.25, im: -0.0625 }));
    assert_eq!(parse_complex(",-0.0625"), None);
}
