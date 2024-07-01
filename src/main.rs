mod gradient;

use clap::Parser;
use gradient::*;
use image::{ImageBuffer, Rgb};
use noise::{NoiseFn, Perlin};
use rand::{
    distributions::{Alphanumeric, DistString},
    Rng,
};
use std::{
    cmp::min,
    fs::File,
    hash::{DefaultHasher, Hash, Hasher},
    io::BufReader,
    ops::DerefMut,
    str::FromStr,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;
const DEFAULT_NOISE_STRENGTH: f64 = 0.25;
const DEFAULT_HEIGHT_OFFSET: f64 = 0.0;
const DEFAULT_OUTPUT: &str = "output.png";

/// Program to generate maps and save them as png images.
#[derive(serde::Serialize, serde::Deserialize, Parser, Clone, Debug)]
#[command(
    version,
    about,
    long_about = None,
)]
struct Config {
    /// Path to configuration JSON file
    #[serde(skip_serializing, skip_deserializing)]
    #[arg(short, long)]
    config_file: Option<String>,

    /// Seed to start generating with
    #[arg(short, long)]
    seed: Option<String>,

    /// Width of image
    #[arg(short, long, value_parser= clap::value_parser!(u32).range(1..))]
    width: Option<u32>,

    /// Height of image
    #[arg(short, long, value_parser= clap::value_parser!(u32).range(1..))]
    height: Option<u32>,

    /// Strength of white noise applied to Perlin noise
    #[arg(short, long, value_parser= noise_strength_in_range)]
    noise_strength: Option<f64>,

    /// Base height at which to start while generating
    #[arg(short, long, value_parser= base_height_in_range)]
    base_height: Option<f64>,

    /// Output path to save image at
    #[arg(short, long)]
    output_path: Option<String>,
}

fn noise_strength_in_range(s: &str) -> Result<f64, String> {
    let noise_strength = s.parse().map_err(|_| format!("{s} is not a number."))?;

    if noise_strength >= 0.0 {
        Ok(noise_strength)
    } else {
        Err(format!("Noise strength must not be negative!"))
    }
}

fn base_height_in_range(s: &str) -> Result<f64, String> {
    let base_height = s.parse().map_err(|_| format!("{s} is not a number."))?;

    if base_height >= 0.0 && base_height <= 1.0 {
        Ok(base_height)
    } else {
        Err(format!("Base height must be between 0 and 1!"))
    }
}

#[allow(dead_code)]
impl Config {
    pub fn merge(&self, other: &Config) -> Self {
        Config {
            width: self.width.or(other.width.or(None)),
            height: self.height.or(other.height.or(None)),
            noise_strength: self.noise_strength.or(other.noise_strength.or(None)),
            base_height: self.base_height.or(other.base_height.or(None)),
            seed: self.seed.clone().or(other.seed.clone().or(None)),
            output_path: self
                .output_path
                .clone()
                .or(other.output_path.clone().or(None)),
            config_file: self
                .config_file
                .clone()
                .or(other.config_file.clone().or(None)),
        }
    }

    pub fn merge_with_defaults(&self, other: &Config) -> Self {
        Config {
            width: self.width.or(other.width.or(Some(DEFAULT_WIDTH))),
            height: self.height.or(other.height.or(Some(DEFAULT_HEIGHT))),
            noise_strength: self
                .noise_strength
                .or(other.noise_strength.or(Some(DEFAULT_NOISE_STRENGTH))),
            base_height: self
                .base_height
                .or(other.base_height.or(Some(DEFAULT_HEIGHT_OFFSET))),
            seed: self.seed.clone().or(other.seed.clone().or(Some(
                Alphanumeric.sample_string(&mut rand::thread_rng(), 32),
            ))),
            output_path: self
                .output_path
                .clone()
                .or(other.output_path.clone().or(Some(DEFAULT_OUTPUT.into()))),
            config_file: self
                .config_file
                .clone()
                .or(other.config_file.clone().or(None)),
        }
    }
}

fn read<T>(default: T) -> T
where
    T: FromStr + Clone,
{
    let mut line = String::new();

    if let Err(err) = std::io::stdin().read_line(&mut line) {
        panic!("{err:?}");
    }
    line = line.trim().into();

    if line.is_empty() {
        default
    } else {
        line.parse::<T>().unwrap_or(default.clone())
    }
}

fn main() -> Result<(), std::io::Error> {
    let config: Config;

    let config_args = Config::parse();

    if let Some(config_path) = &config_args.config_file {
        let path = std::path::PathBuf::from(config_path);

        if path.exists() {
            let config_file = File::open(path)?;
            let config_reader = BufReader::new(config_file);
            let config_json: Config = serde_json::from_reader(config_reader).map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("{err:?}"))
            })?;

            config = config_args.merge_with_defaults(&config_json);
        } else {
            println!("Provided config file does not exist: '{path:?}'");
            config = config_args.merge_with_defaults(&config_args);
        }
    } else {
        config = config_args.merge_with_defaults(&config_args);
    }

    println!("{config:?}");

    let mut hasher = DefaultHasher::new();

    config.seed.hash(&mut hasher);
    let seed = hasher.finish();
    let width = config.width.unwrap();
    let height = config.height.unwrap();
    let output_path = config.output_path.as_ref().unwrap();

    let gradient = Gradient::default();
    let perlin = Perlin::new(seed as u32);

    let mut steps: [f64; SCALES.len()] = [0.0; SCALES.len()];
    for (idx, scale) in SCALES.iter().enumerate() {
        let step_x = *scale as f64 / width as f64;
        let step_y = *scale as f64 / height as f64;
        steps[idx] = f64::min(step_x, step_y);
    }

    let progress_step = 100.0 / (width as f64 * height as f64);
    let progress = Arc::new(Mutex::new(0.0));

    let mut image_data: Vec<u8> = vec![0; (width * height * 3) as usize];
    let image_slice = &mut image_data[..];

    let area_size: usize = (width * height) as usize / (num_cpus::get() - 1);

    let progress_clone = progress.clone();

    let output_thread = std::thread::spawn(move || loop {
        match progress_clone.try_lock() {
            Ok(progress) => {
                print!("Generating: {progress:.3}\r");
                if *progress >= 100.0 {
                    break;
                }
            }
            Err(std::sync::TryLockError::WouldBlock) => {}
            Err(err) => panic!("{err:?}"),
        }
        sleep(Duration::from_nanos(100));
    });

    let _ = crossbeam::scope(|scope| {
        for (area, slice) in image_slice.chunks_mut(area_size * 3).enumerate() {
            let perlin = perlin.clone();
            let gradient = gradient.clone();
            let config = config.clone();
            let progress = progress.clone();
            scope.spawn(move |_| {
                job(
                    slice,
                    area * area_size,
                    area_size,
                    &steps,
                    perlin,
                    gradient,
                    config,
                    progress,
                    progress_step,
                )
            });
        }
    });

    *progress.lock().unwrap() = 100.0;
    output_thread.join().unwrap();

    println!("Done!");
    println!("Writing output to: {output_path}");

    let image: ImageBuffer<Rgb<u8>, Vec<u8>> =
        match ImageBuffer::from_vec(width, height, image_data) {
            Some(image) => image,
            None => panic!("Could not create image!"),
        };

    image
        .save(output_path)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

    Ok(())
}

fn job(
    image: &mut [u8],
    start: usize,
    amount: usize,
    steps: &[f64; SCALES.len()],
    perlin: Perlin,
    gradient: Gradient,
    config: Config,
    progress: Arc<Mutex<f64>>,
    progress_step: f64,
) {
    let width = config.width.unwrap() as usize;
    let mut min = f64::MAX;
    let mut max = f64::MIN;
    for idx in 0..std::cmp::min(image.len() / 3, amount) {
        let x = (start + idx) % width;
        let y = (start + idx) / width;
        let mut value: f64 = 0.0;
        for layer_idx in 0..SCALES.len() {
            let step = steps[layer_idx];
            let x = step * x as f64;
            let y = step * y as f64;
            value += perlin.get([x, y]) * WEIGHTS[layer_idx];
        }

        value += 0.5;

        if value > max {
            max = value;
        }

        if value < min {
            min = value;
        }

        let base_height = config.base_height.unwrap();
        let noise_value = rand::thread_rng().gen_range(0..100) as f64 / 10000.0;

        // map value to be inside valid range
        value = base_height
            + value * (1.0 - base_height)
            // and apply noise
            + noise_value * config.noise_strength.unwrap();

        // limit values to be within range
        value = value.clamp(0.0000001, 0.99999999);

        let color = gradient.lerp_color(value).0;
        image[idx * 3] = color[0];
        image[idx * 3 + 1] = color[1];
        image[idx * 3 + 2] = color[2];
        match progress.lock() {
            Ok(mut progress) => {
                *progress.deref_mut() += progress_step;
            }
            Err(err) => panic!("{err:?}"),
        }
    }

    println!("Min {min:.3}, Max {max:.3}");
}
