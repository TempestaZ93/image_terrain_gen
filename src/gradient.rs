use std::collections::HashMap;

use rand::Rng;

pub const SCALES: [f64; 5] = [2.0, 4.0, 8.0, 16.0, 32.0];
pub const WEIGHTS: [f64; 5] = [0.5, 0.3, 0.1, 0.05, 0.05];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TerrainKind {
    Undefined = -1,
    DeepWater = 0,
    Water = 1,
    ShallowWater = 2,
    Shore = 3,
    FlatLand = 4,
    HighLand = 5,
    Mountains = 6,
    MountainTop,
}

impl TerrainKind {
    pub fn before(&self) -> Self {
        match self {
            TerrainKind::Undefined => TerrainKind::Undefined,
            TerrainKind::DeepWater => TerrainKind::DeepWater,
            TerrainKind::Water => TerrainKind::DeepWater,
            TerrainKind::ShallowWater => TerrainKind::Water,
            TerrainKind::Shore => TerrainKind::Water,
            TerrainKind::FlatLand => TerrainKind::Shore,
            TerrainKind::HighLand => TerrainKind::FlatLand,
            TerrainKind::Mountains => TerrainKind::HighLand,
            TerrainKind::MountainTop => TerrainKind::Mountains,
        }
    }

    pub fn after(&self) -> Self {
        match self {
            TerrainKind::Undefined => TerrainKind::Undefined,
            TerrainKind::DeepWater => TerrainKind::Water,
            TerrainKind::Water => TerrainKind::ShallowWater,
            TerrainKind::ShallowWater => TerrainKind::Shore,
            TerrainKind::Shore => TerrainKind::FlatLand,
            TerrainKind::FlatLand => TerrainKind::HighLand,
            TerrainKind::HighLand => TerrainKind::Mountains,
            TerrainKind::Mountains => TerrainKind::MountainTop,
            TerrainKind::MountainTop => TerrainKind::MountainTop,
        }
    }
}

#[derive(Clone)]
pub struct Gradient {
    pub terrain_limits: HashMap<TerrainKind, (f64, f64)>,
    pub colors: HashMap<TerrainKind, image::Rgb<u8>>,
}

#[allow(dead_code, unused)]
impl Gradient {
    pub fn new(
        terrain_limits: HashMap<TerrainKind, (f64, f64)>,
        colors: HashMap<TerrainKind, image::Rgb<u8>>,
    ) -> Self {
        Self {
            terrain_limits,
            colors,
        }
    }

    pub fn get_color(&self, height: f64) -> image::Rgb<u8> {
        let kind = self.get_terrain_kind(height);
        self.colors[&kind]
    }

    pub fn lerp_noise_color(&self, height: f64, noise_strength: Option<f64>) -> image::Rgb<u8> {
        // add noise offset to break up even noise
        let height = height
            + (rand::thread_rng().gen_range(0..200) as f64 / 10000.0 - 0.001) / 4.0
                * noise_strength.unwrap_or(1.0);
        let height = height.clamp(0.0, 1.0);

        self.lerp_color(height)
    }

    pub fn lerp_color(&self, height: f64) -> image::Rgb<u8> {
        let kind = self.get_terrain_kind(height);
        let kind_before = kind.before();
        let kind_after = kind.after();

        let color = self.colors[kind];
        let color_before = self.colors[&kind_before];
        let color_after = self.colors[&kind_after];

        let dist_self = height - self._terrain_center(*kind);
        let dist_self_abs = dist_self.abs();
        let dist_before = (height - self._terrain_center(kind_before)).abs();
        let dist_after = (height - self._terrain_center(kind_after)).abs();

        if dist_self < 0.0 {
            // height is closer to before
            let factor_before = dist_self_abs / (dist_before + dist_self_abs);
            Gradient::_lerp_colors(color_before, factor_before, color)
        } else {
            // height is closer to after
            let factor_after = dist_self_abs / (dist_after + dist_self_abs);
            Gradient::_lerp_colors(color_after, factor_after, color)
        }
    }

    pub fn get_terrain_kind(&self, height: f64) -> &TerrainKind {
        assert!(height >= 0.0 && height <= 1.0);
        if let Some(kind) = self
            .terrain_limits
            .iter()
            .find(|(_, (min, max))| height > *min && height < *max)
            .map(|(kind, _)| kind)
        {
            kind
        } else {
            println!("{height}");
            &TerrainKind::Undefined
        }
    }

    fn _terrain_center(&self, kind: TerrainKind) -> f64 {
        let limits = self.terrain_limits[&kind];
        (limits.0 + limits.1) / 2.0
    }

    fn _lerp_colors(one: image::Rgb<u8>, factor: f64, other: image::Rgb<u8>) -> image::Rgb<u8> {
        let factor_inverse = 1.0 - factor;
        let image::Rgb(one_rgb) = one;
        let image::Rgb(other_rgb) = other;
        let r = (one_rgb[0] as f64 * factor + other_rgb[0] as f64 * factor_inverse) as u8;
        let g = (one_rgb[1] as f64 * factor + other_rgb[1] as f64 * factor_inverse) as u8;
        let b = (one_rgb[2] as f64 * factor + other_rgb[2] as f64 * factor_inverse) as u8;
        image::Rgb([r, g, b])
    }
}

impl Default for Gradient {
    fn default() -> Self {
        let mut terrain_limits = HashMap::new();
        let mut colors = HashMap::new();

        terrain_limits.insert(TerrainKind::DeepWater, (0.0, 0.4));
        terrain_limits.insert(TerrainKind::Water, (0.4, 0.6));
        terrain_limits.insert(TerrainKind::ShallowWater, (0.6, 0.66));
        terrain_limits.insert(TerrainKind::Shore, (0.66, 0.67));
        terrain_limits.insert(TerrainKind::FlatLand, (0.67, 0.8));
        terrain_limits.insert(TerrainKind::HighLand, (0.8, 0.9));
        terrain_limits.insert(TerrainKind::Mountains, (0.9, 0.95));
        terrain_limits.insert(TerrainKind::MountainTop, (0.95, 1.0));

        colors.insert(TerrainKind::DeepWater, image::Rgb([0, 64, 106]));
        colors.insert(TerrainKind::Water, image::Rgb([0, 117, 119]));
        colors.insert(TerrainKind::ShallowWater, image::Rgb([180, 240, 251]));
        colors.insert(TerrainKind::Shore, image::Rgb([194, 178, 128]));
        colors.insert(TerrainKind::FlatLand, image::Rgb([72, 111, 56]));
        colors.insert(TerrainKind::HighLand, image::Rgb([111, 130, 70]));
        colors.insert(TerrainKind::Mountains, image::Rgb([79, 79, 79]));
        colors.insert(TerrainKind::MountainTop, image::Rgb([253, 254, 255]));

        colors.insert(TerrainKind::Undefined, image::Rgb([255, 0, 255]));

        Self {
            terrain_limits,
            colors,
        }
    }
}
