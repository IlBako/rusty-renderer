use bytemuck::{Pod, Zeroable};
use glam::Vec3;

pub(crate) struct Light {
    color: Vec3,
    direction: Vec3,
    intensity: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct GpuLight {
    color: [f32; 3],
    intensity: f32,
    direction: [f32; 3],
    _padding1: f32,
}

const _: () = assert!(size_of::<GpuLight>() == 32);

impl Light {
    pub fn new(color: Vec3, intensity: f32, direction: Vec3) -> Self {
        let norm_dir = direction
            .try_normalize()
            .expect("Error normalizing light direction, NaN returned");
        Self {
            color,
            intensity,
            direction: norm_dir,
        }
    }

    pub(crate) fn to_gpu(&self) -> GpuLight {
        GpuLight {
            color: self.color.to_array(),
            intensity: self.intensity,
            direction: self.direction.to_array(),
            _padding1: 0.0,
        }
    }
}
