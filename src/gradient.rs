pub const SCALES: [f64; 7] = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0];
pub const WEIGHTS: [f64; 7] = [0.35, 0.2, 0.15, 0.075, 0.075, 0.025, 0.025];

///
/// Levels of terrain representing different heights and associated colors
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TerrainKind {
    Undefined = -1,
    DeepWater = 0,
    Water,
    ShallowWater,
    Shore,
    FlatLand,
    HighLand,
    Mountains,
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

impl From<usize> for TerrainKind {
    fn from(value: usize) -> Self {
        match value {
            0 => TerrainKind::DeepWater,
            1 => TerrainKind::Water,
            2 => TerrainKind::ShallowWater,
            3 => TerrainKind::Shore,
            4 => TerrainKind::FlatLand,
            5 => TerrainKind::HighLand,
            6 => TerrainKind::Mountains,
            7 => TerrainKind::MountainTop,
            _ => TerrainKind::Undefined,
        }
    }
}

#[derive(Clone)]
pub struct Gradient {
    pub terrain_limits: Vec<(f64, f64)>,
    pub terrain_centers: Vec<f64>,
    pub colors: Vec<image::Rgb<u8>>,
}

#[allow(dead_code, unused)]
impl Gradient {
    pub fn new(terrain_limits: Vec<(f64, f64)>, colors: Vec<image::Rgb<u8>>) -> Self {
        let terrain_centers = Gradient::calc_centers(&terrain_limits);
        Self {
            terrain_limits,
            terrain_centers,
            colors,
        }
    }

    pub fn get_color(&self, height: f64) -> image::Rgb<u8> {
        let kind = self.get_terrain_kind(height).unwrap();
        self.colors[kind as usize]
    }

    pub fn lerp_color(&self, height: f64) -> image::Rgb<u8> {
        let kind = self.get_terrain_kind(height).unwrap();
        let kind_before = kind.before();
        let kind_after = kind.after();

        let color = self.colors[kind as usize];
        let color_before = self.colors[kind_before as usize];
        let color_after = self.colors[kind_after as usize];

        let dist_self = height - self._terrain_center(kind);
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

    pub fn get_terrain_kind(&self, height: f64) -> Result<TerrainKind, ()> {
        if let Some(kind) = self
            .terrain_limits
            .iter()
            .position(|((min, max))| height > *min && height < *max)
        {
            Ok(TerrainKind::from(kind))
        } else {
            println!("{height}");
            Err(())
        }
    }

    fn _terrain_center(&self, kind: TerrainKind) -> f64 {
        let (min, max) = self.terrain_limits[kind as usize];
        (min + max) / 2.0
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

    fn calc_centers(terrain_limits: &Vec<(f64, f64)>) -> Vec<f64> {
        let terrain_centers = Vec::new();

        terrain_centers
    }
}

impl Default for Gradient {
    fn default() -> Self {
        let mut terrain_limits = vec![(0.0, 0.0); TerrainKind::MountainTop as usize + 1];
        let mut colors = vec![image::Rgb::<u8>([0, 0, 0]); TerrainKind::MountainTop as usize + 1];

        terrain_limits[TerrainKind::DeepWater as usize] = (0.0, 0.4);
        terrain_limits[TerrainKind::Water as usize] = (0.4, 0.6);
        terrain_limits[TerrainKind::ShallowWater as usize] = (0.6, 0.63);
        terrain_limits[TerrainKind::Shore as usize] = (0.63, 0.64);
        terrain_limits[TerrainKind::FlatLand as usize] = (0.64, 0.8);
        terrain_limits[TerrainKind::HighLand as usize] = (0.8, 0.9);
        terrain_limits[TerrainKind::Mountains as usize] = (0.9, 0.98);
        terrain_limits[TerrainKind::MountainTop as usize] = (0.98, 1.0);

        colors[TerrainKind::DeepWater as usize] = image::Rgb([0, 64, 106]);
        colors[TerrainKind::Water as usize] = image::Rgb([0, 117, 119]);
        colors[TerrainKind::ShallowWater as usize] = image::Rgb([180, 240, 251]);
        colors[TerrainKind::Shore as usize] = image::Rgb([194, 178, 128]);
        colors[TerrainKind::FlatLand as usize] = image::Rgb([72, 111, 56]);
        colors[TerrainKind::HighLand as usize] = image::Rgb([111, 130, 70]);
        colors[TerrainKind::Mountains as usize] = image::Rgb([79, 79, 79]);
        colors[TerrainKind::MountainTop as usize] = image::Rgb([253, 254, 255]);

        Gradient::new(terrain_limits, colors)
    }
}
