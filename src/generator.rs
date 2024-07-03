use crate::gradient::*;

use noise::{NoiseFn, Perlin};
use rand::Rng;

pub fn generate(
    seed: u64,
    width: u32,
    height: u32,
    base_level: f64,
    noise_strength: f64,
    thread_count: Option<usize>,
    image_data: &mut Vec<u8>,
) {
    let gradient = Gradient::default();
    let perlin = Perlin::new(seed as u32);

    let thread_count = thread_count.unwrap_or(num_cpus::get() - 1);

    let mut steps: [f64; SCALES.len()] = [0.0; SCALES.len()];
    for (idx, scale) in SCALES.iter().enumerate() {
        let step_x = *scale as f64 / width as f64;
        let step_y = *scale as f64 / height as f64;
        steps[idx] = f64::min(step_x, step_y);
    }

    let image_slice = &mut image_data[..];

    let area_size = (width * height) as usize / thread_count;

    let _ = crossbeam::scope(|scope| {
        for (area, slice) in image_slice.chunks_mut(area_size * 3).enumerate() {
            let perlin = perlin.clone();
            let gradient = gradient.clone();
            scope.spawn(move |_| {
                job(
                    slice,
                    area * area_size,
                    area_size,
                    &steps,
                    perlin,
                    gradient,
                    width as usize,
                    base_level,
                    noise_strength,
                )
            });
        }
    });
}

fn job(
    image: &mut [u8],
    start: usize,
    amount: usize,
    steps: &[f64; SCALES.len()],
    perlin: Perlin,
    gradient: Gradient,
    width: usize,
    base_level: f64,
    noise_strength: f64,
) {
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

        let noise_value = rand::thread_rng().gen_range(0..1000) as f64 / 100000.0;

        // map value to be inside valid range
        value = base_level
            + value * (1.0 - base_level)
            // and apply noise
            + noise_value * noise_strength;

        // limit values to be within range
        value = value.clamp(0.0000001, 0.99999999);

        let [r, g, b] = gradient.lerp_color(value).0;
        image[idx * 3] = r;
        image[idx * 3 + 1] = g;
        image[idx * 3 + 2] = b;
    }
}
