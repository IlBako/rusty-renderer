use bytemuck::{Pod, Zeroable};
use glam::Vec3;

pub struct Material {
    albedo: Vec3,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuMaterial {
    albedo: [f32; 3],
    _padding: f32,
}

const _: () = assert!(size_of::<GpuMaterial>() == 16);

impl Material {
    pub fn new(albedo: Vec3) -> Self {
        Self { albedo }
    }

    pub(crate) fn to_gpu(&self) -> GpuMaterial {
        GpuMaterial {
            albedo: self.albedo.to_array(),
            _padding: 0.0,
        }
    }
}
