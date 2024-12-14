mod config;
mod hasher;

use std::{
    hash::{Hash, Hasher},
    time::Instant,
};

use config::*;
use hasher::SeedHasher;
use image::{ImageBuffer, Rgb};
use map_generation::generator;

fn main() -> Result<(), std::io::Error> {
    let config = Config::new().unwrap();
    if config.dump_config.unwrap() {
        println!("{config}");
    }

    let width = config.width.unwrap();
    let height = config.height.unwrap();
    let output_path = config.output_path.as_ref().unwrap();
    let verbose = config.verbose.unwrap();

    let mut image: Vec<u8> = vec![0; (width * height * 3) as usize];

    if verbose {
        println!("Generating...");
    }

    let mut hasher = SeedHasher::new();
    config.seed.unwrap().hash(&mut hasher);

    let start = Instant::now();
    generator::generate(
        &mut image,
        hasher.finish(),
        config.width.unwrap(),
        config.height.unwrap(),
        config.base_level.unwrap(),
        config.noise_strength.unwrap(),
        None,
        None,
    );
    let end = Instant::now();
    let duration = end - start;

    if verbose {
        println!(
            "Done! Took {:.3?} (~ {} px / sec)",
            end - start,
            // calculate number of pixels per second
            (((width * height) as f64 / duration.as_secs_f64()) as u64)
                // format result as comma split 1000s like in this number: 10,000,000
                .to_string()
                .as_bytes()
                .rchunks(3)
                .rev()
                .map(std::str::from_utf8)
                .collect::<Result<Vec<&str>, _>>()
                .unwrap()
                .join(",")
        );
        println!("Writing output to: {output_path}");
    }

    let image: ImageBuffer<Rgb<u8>, Vec<u8>> =
        match ImageBuffer::from_vec(config.width.unwrap(), config.height.unwrap(), image) {
            Some(image) => image,
            None => panic!("Could not create image from vector data!"),
        };

    image
        .save(output_path)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

    Ok(())
}
