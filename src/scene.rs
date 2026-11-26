use bytemuck::{Pod, Zeroable};
use glam::Vec3;

use crate::material::{GpuMaterial, Material};

pub struct Sphere {
    center: Vec3,
    radius: f32,
    material_id: MaterialId,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuSphere {
    center: [f32; 3],
    radius: f32,
    material_idx: u32,
    _padding: [f32; 3],
}

const _: () = assert!(size_of::<GpuSphere>() == 32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialId(u32);

impl MaterialId {
    pub(crate) fn index(self) -> u32 {
        self.0
    }
}

pub struct Scene {
    materials: Vec<Material>,
    spheres: Vec<Sphere>,
}

impl Scene {
    fn new() -> Self {
        Self {
            materials: Vec::new(),
            spheres: Vec::new(),
        }
    }

    pub fn demo() -> Self {
        let mut scene = Scene::new();

        let white = scene.add_material(Material::new(srgb_to_linear(Vec3::new(
            255.0, 255.0, 255.0,
        ))));
        let ground =
            scene.add_material(Material::new(srgb_to_linear(Vec3::new(165.0, 42.0, 42.0))));
        let purple =
            scene.add_material(Material::new(srgb_to_linear(Vec3::new(128.0, 0.0, 255.0))));
        // let ocra = scene.add_material(Material::new(srgb_to_linear(Vec3::new(209.0, 186.0, 56.0))));

        scene.add_sphere(Vec3::new(0.0, 0.0, -3.0), 1.0, white);
        scene.add_sphere(Vec3::new(0.0, -101.0, -3.0), 100.0, ground);
        scene.add_sphere(Vec3::new(-2.2, 0.0, -3.5), 1.0, purple);
        scene.add_sphere(Vec3::new(2.2, 0.0, -3.5), 1.0, purple);

        scene
    }

    pub fn get_gpu_spheres(&self) -> Vec<GpuSphere> {
        self.spheres.iter().map(Sphere::to_gpu).collect()
    }

    pub fn get_gpu_materials(&self) -> Vec<GpuMaterial> {
        self.materials.iter().map(Material::to_gpu).collect()
    }

    pub fn add_material(&mut self, material: Material) -> MaterialId {
        let id = u32::try_from(self.materials.len()).expect("Material array length overflows u32");
        self.materials.push(material);
        MaterialId(id)
    }

    pub fn add_sphere(&mut self, center: Vec3, radius: f32, material_id: MaterialId) {
        self.spheres.push(Sphere {
            center,
            radius,
            material_id,
        });
    }
}

impl Sphere {
    pub(crate) fn to_gpu(&self) -> GpuSphere {
        GpuSphere {
            center: self.center.to_array(),
            radius: self.radius,
            material_idx: self.material_id.index(),
            _padding: [0.0, 0.0, 0.0],
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
