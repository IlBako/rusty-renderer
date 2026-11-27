use glam::Vec3;

pub(crate) struct Material {
    albedo: Vec3,
}

impl Material {
    pub(super) fn new(albedo: Vec3) -> Self {
        Self { albedo }
    }

    pub(crate) fn albedo(&self) -> Vec3 {
        self.albedo
    }
}
