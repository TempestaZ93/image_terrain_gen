use std::{fmt::Display, fs::File, io::BufReader};

use clap::Parser;
use rand::distributions::{Alphanumeric, DistString};

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;
const DEFAULT_NOISE_STRENGTH: f64 = 0.25;
const DEFAULT_HEIGHT_OFFSET: f64 = 0.0;
const DEFAULT_OUTPUT: &str = "output.png";
const DEFAULT_DUMP_CONFIG: bool = false;

/// Program to generate maps and save them as png images.
#[derive(serde::Serialize, serde::Deserialize, Parser, Clone, Debug)]
#[command(
    version,
    about,
    long_about = None,
)]
pub struct Config {
    /// Path to configuration JSON file
    #[serde(skip_deserializing)]
    #[arg(short = 'i', long)]
    pub config_file: Option<String>,

    /// Output path to save image at
    #[serde(skip_serializing)]
    #[arg(short, long)]
    pub dump_config: Option<bool>,

    /// Seed to start generating with
    #[arg(short, long)]
    pub seed: Option<String>,

    /// Width of image
    #[arg(short, long, value_parser= clap::value_parser!(u32).range(1..))]
    pub width: Option<u32>,

    /// Height of image
    #[arg(short, long, value_parser= clap::value_parser!(u32).range(1..))]
    pub height: Option<u32>,

    /// Strength of white noise applied to Perlin noise
    #[arg(short, long, value_parser= noise_strength_in_range)]
    pub noise_strength: Option<f64>,

    /// Base height at which to start while generating
    #[arg(short, long, value_parser= base_height_in_range)]
    pub base_height: Option<f64>,

    /// Number of threads created to generate image
    #[arg(short='j', long, value_parser= thread_count_in_range)]
    pub thread_count: Option<usize>,

    /// Output path to save image at
    #[arg(short, long)]
    pub output_path: Option<String>,
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

fn thread_count_in_range(s: &str) -> Result<usize, String> {
    let cpu_count = s.parse().map_err(|_| format!("{s} is not a number."))?;

    if cpu_count >= 1 && cpu_count <= 256 {
        Ok(cpu_count)
    } else {
        Err(format!("Noise strength must not be negative!"))
    }
}

#[allow(dead_code)]
impl Config {
    pub fn new() -> Result<Self, std::io::Error> {
        let config: Config;

        let config_args = Config::parse();

        if let Some(config_path) = &config_args.config_file {
            let path = std::path::PathBuf::from(config_path);

            if path.exists() {
                let config_file = File::open(path)?;
                let config_reader = BufReader::new(config_file);
                let config_json: Config =
                    serde_json::from_reader(config_reader).map_err(|err| {
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

        Ok(config)
    }

    fn merge(&self, other: &Config) -> Self {
        Config {
            dump_config: self
                .dump_config
                .or(other.dump_config.or(Some(DEFAULT_DUMP_CONFIG))),
            width: self.width.or(other.width.or(None)),
            height: self.height.or(other.height.or(None)),
            noise_strength: self.noise_strength.or(other.noise_strength.or(None)),
            base_height: self.base_height.or(other.base_height.or(None)),
            seed: self.seed.clone().or(other.seed.clone().or(None)),
            thread_count: self.thread_count.or(other.thread_count.or(None)),
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

    fn merge_with_defaults(&self, other: &Config) -> Self {
        Config {
            dump_config: self
                .dump_config
                .or(other.dump_config.or(Some(DEFAULT_DUMP_CONFIG))),
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
            thread_count: self
                .thread_count
                .or(other.thread_count.or(Some(num_cpus::get() - 1))),
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

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
