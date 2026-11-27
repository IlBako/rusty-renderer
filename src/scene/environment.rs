use glam::Vec3;

use super::color::srgb_to_linear;

pub(crate) struct Environment {
    sky_color: Vec3,
    ambient: f32,
}

impl Environment {
    pub(super) fn new(sky_color: Vec3, ambient: f32) -> Self {
        Self { sky_color, ambient }
    }

    pub(crate) fn sky_color(&self) -> Vec3 {
        self.sky_color
    }

    pub(crate) fn ambient(&self) -> f32 {
        self.ambient
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            sky_color: srgb_to_linear(Vec3::new(82.0, 209.0, 234.0)),
            ambient: 0.05,
        }
    }
}
