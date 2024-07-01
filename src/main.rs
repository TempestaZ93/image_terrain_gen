pub mod config;
pub mod generator;
pub mod gradient;

use config::*;
use generator::*;

use image::{ImageBuffer, Rgb};

fn main() -> Result<(), std::io::Error> {
    let config = Config::new().unwrap();
    if config.dump_config.unwrap() {
        println!("{config}");
    }

    let width = config.width.unwrap();
    let height = config.height.unwrap();
    let output_path = config.output_path.as_ref().unwrap();

    let generator = Generator::new(&config);

    let mut image_data: Vec<u8> = vec![0; (width * height * 3) as usize];

    println!("Generating...");
    let start = std::time::Instant::now();
    generator.generate(&mut image_data);
    let end = std::time::Instant::now();
    let duration = end - start;
    println!(
        "Done! Took {:?} (~ {} px / sec)",
        end - start,
        (((width * height) as f64 / duration.as_secs_f64()) as u64)
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

    let image: ImageBuffer<Rgb<u8>, Vec<u8>> =
        match ImageBuffer::from_vec(config.width.unwrap(), config.height.unwrap(), image_data) {
            Some(image) => image,
            None => panic!("Could not create image!"),
        };

    image
        .save(output_path)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

    Ok(())
}
