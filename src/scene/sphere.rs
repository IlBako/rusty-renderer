use glam::Vec3;

use super::MaterialId;

pub(crate) struct Sphere {
    center: Vec3,
    radius: f32,
    material_id: MaterialId,
}

impl Sphere {
    pub(super) fn new(center: Vec3, radius: f32, material_id: MaterialId) -> Self {
        Self {
            center,
            radius,
            material_id,
        }
    }

    pub(crate) fn center(&self) -> Vec3 {
        self.center
    }

    pub(crate) fn radius(&self) -> f32 {
        self.radius
    }

    /// Index of this sphere's material in the scene's material list.
    pub(crate) fn material_index(&self) -> u32 {
        self.material_id.index()
    }
}
