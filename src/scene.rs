use bytemuck::{Pod, Zeroable};
use glam::Vec3;

use crate::{
    light::{GpuLight, Light},
    material::{GpuMaterial, Material},
    rng,
};

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

pub struct Environment {
    sky_color: Vec3,
    ambient: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuSceneSettings {
    sky_color: [f32; 3],
    ambient: f32,
    sphere_count: u32,
    light_count: u32,
    _padding: [f32; 2],
}

const _: () = assert!(size_of::<GpuSceneSettings>() == 32);

impl Environment {
    pub(crate) fn new(sky_color: Vec3, ambient: f32) -> Self {
        Self { sky_color, ambient }
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

pub struct Scene {
    materials: Vec<Material>,
    spheres: Vec<Sphere>,
    lights: Vec<Light>,
    environment: Environment,
}

impl Scene {
    fn new() -> Self {
        Self {
            materials: Vec::new(),
            spheres: Vec::new(),
            lights: Vec::new(),
            environment: Environment::default(),
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

        scene.add_sphere(Vec3::new(0.0, 0.0, -3.0), 1.0, white);
        scene.add_sphere(Vec3::new(0.0, -101.0, -3.0), 100.0, ground);
        scene.add_sphere(Vec3::new(-2.2, 0.0, -3.5), 1.0, purple);
        scene.add_sphere(Vec3::new(2.2, 0.0, -3.5), 1.0, purple);

        scene.add_light(Light::new(
            Vec3::new(1.0, 0.95, 0.9),
            0.5,
            Vec3::new(0.0, 3.0, 3.0),
        ));

        scene.add_light(Light::new(
            Vec3::new(1.0, 0.95, 0.9),
            0.5,
            Vec3::new(-3.0, 2.0, 0.0),
        ));

        scene.environment = Environment::new(srgb_to_linear(Vec3::new(82.0, 209.0, 234.0)), 0.05);

        scene
    }

    pub fn get_gpu_spheres(&self) -> Vec<GpuSphere> {
        self.spheres.iter().map(Sphere::to_gpu).collect()
    }

    pub fn get_gpu_materials(&self) -> Vec<GpuMaterial> {
        self.materials.iter().map(Material::to_gpu).collect()
    }

    pub fn get_gpu_lights(&self) -> Vec<GpuLight> {
        self.lights.iter().map(Light::to_gpu).collect()
    }

    pub fn get_gpu_scene_settings(&self) -> GpuSceneSettings {
        GpuSceneSettings {
            sky_color: self.environment.sky_color.to_array(),
            ambient: self.environment.ambient,
            sphere_count: self.get_sphere_count(),
            light_count: self.get_light_count(),
            _padding: [0.0, 0.0],
        }
    }

    pub fn get_sphere_count(&self) -> u32 {
        u32::try_from(self.spheres.len()).expect("Spheres count overflows u32::MAX")
    }

    pub fn get_material_count(&self) -> u32 {
        u32::try_from(self.materials.len()).expect("Materials count overflows u32::MAX")
    }

    pub fn get_light_count(&self) -> u32 {
        u32::try_from(self.lights.len()).expect("Lights count overflows u32::MAX")
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

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn add_random_sphere(&mut self, rng: &mut rng::Rng) {
        if self.get_material_count() == 0 {
            return;
        }

        let center = rng.next_vec3((Vec3::new(-5.0, 0.0, -5.0), Vec3::new(5.0, 10.0, 5.0)));
        let radius = rng.next_f32_range(0.5, 2.0);
        let material_id = rng.next_u32_range(0, self.get_material_count());
        self.add_sphere(center, radius, MaterialId(material_id));
    }

    pub fn remove_last_sphere(&mut self) {
        self.spheres.pop();
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
