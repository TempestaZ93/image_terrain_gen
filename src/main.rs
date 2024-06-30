mod gradient;

use gradient::*;
use image::{ImageBuffer, Rgb};
use noise::{NoiseFn, Perlin};
use rand::distributions::{Alphanumeric, DistString};
use std::{
    cmp::min,
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    ops::DerefMut,
    str::FromStr,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;
const DEFAULT_OUTPUT: &str = "output.png";

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
    let mut hasher = DefaultHasher::new();

    print!("Seed (random) >> ");
    std::io::stdout().flush()?;
    let seed_str: String = read(Alphanumeric.sample_string(&mut rand::thread_rng(), 32));
    seed_str.hash(&mut hasher);
    let seed = hasher.finish();

    print!("Width ({}) >> ", DEFAULT_WIDTH);
    std::io::stdout().flush()?;
    let width: u32 = read(DEFAULT_WIDTH);

    print!("Height ({}) >> ", DEFAULT_HEIGHT);
    std::io::stdout().flush()?;
    let height: u32 = read(DEFAULT_HEIGHT);

    print!("Output ({}) >> ", DEFAULT_OUTPUT);
    std::io::stdout().flush()?;
    let output_path: String = read(DEFAULT_OUTPUT.into());

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
            let progress = progress.clone();
            scope.spawn(move |_| {
                job(
                    slice,
                    width as usize,
                    area * area_size,
                    area_size,
                    &steps,
                    perlin,
                    gradient,
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
    width: usize,
    start: usize,
    amount: usize,
    steps: &[f64; SCALES.len()],
    perlin: Perlin,
    gradient: Gradient,
    progress: Arc<Mutex<f64>>,
    progress_step: f64,
) {
    for idx in 0..min(image.len() / 3, amount) {
        let x = (start + idx) % width;
        let y = (start + idx) / width;
        let mut value: f64 = 0.0;
        for layer_idx in 0..SCALES.len() {
            let step = steps[layer_idx];
            let x = step * x as f64;
            let y = step * y as f64;
            value += perlin.get([x, y]) * WEIGHTS[layer_idx];
        }

        value += 1.0;
        value /= 2.0;
        value = value.clamp(0.0, 1.0);

        let color = gradient.lerp_noise_color(value, None).0;
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
}
