use glam::Vec3;

pub(crate) struct Light {
    color: Vec3,
    direction: Vec3,
    intensity: f32,
}

impl Light {
    pub(super) fn new(color: Vec3, intensity: f32, direction: Vec3) -> Self {
        let norm_dir = direction
            .try_normalize()
            .expect("Error normalizing light direction, NaN returned");
        Self {
            color,
            intensity,
            direction: norm_dir,
        }
    }

    pub(crate) fn color(&self) -> Vec3 {
        self.color
    }

    pub(crate) fn direction(&self) -> Vec3 {
        self.direction
    }

    pub(crate) fn intensity(&self) -> f32 {
        self.intensity
    }
}
