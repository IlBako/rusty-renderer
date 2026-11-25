use bytemuck::{Pod, Zeroable};
use glam::Vec3;

pub struct Sphere {
    center: Vec3,
    radius: f32,
    color: Vec3,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuSphere {
    center: [f32; 3],
    radius: f32,
    color: [f32; 3],
    _padding: f32,
}

const _: () = assert!(size_of::<GpuSphere>() == 32);

pub struct Scene {
    spheres: Vec<Sphere>,
}

impl Scene {
    pub fn demo() -> Self {
        let mut spheres = Vec::new();
        spheres.push(Sphere {
            center: Vec3::new(0.0, 0.0, -3.0),
            radius: 1.0,
            color: srgb_to_linear(Vec3::new(255.0, 255.0, 255.0)),
        });
        spheres.push(Sphere {
            center: Vec3::new(0.0, -101.0, -3.0),
            radius: 100.0,
            color: srgb_to_linear(Vec3::new(165.0, 42.0, 42.0)),
        });
        spheres.push(Sphere {
            center: Vec3::new(-2.2, 0.0, -3.5),
            radius: 1.0,
            color: srgb_to_linear(Vec3::new(128.0, 0.0, 255.0)),
        });
        spheres.push(Sphere {
            center: Vec3::new(2.2, 0.0, -3.5),
            radius: 1.0,
            color: srgb_to_linear(Vec3::new(209.0, 186.0, 56.0)),
        });

        Self { spheres }
    }

    pub fn get_gpu_spheres(&self) -> Vec<GpuSphere> {
        self.spheres.iter().map(Sphere::to_gpu).collect()
    }
}

impl Sphere {
    pub(crate) fn to_gpu(&self) -> GpuSphere {
        GpuSphere {
            center: self.center.to_array(),
            radius: self.radius,
            color: self.color.to_array(),
            _padding: 0.0,
        }
    }
}

fn srgb_to_linear(color: Vec3) -> Vec3 {
    fn channel(c: f32) -> f32 {
        let s = c / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }

    Vec3::new(channel(color.x), channel(color.y), channel(color.z))
}
