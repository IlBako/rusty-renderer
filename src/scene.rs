mod camera;
mod color;
mod environment;
mod light;
mod material;
mod sphere;

pub(crate) use camera::Camera;
pub(crate) use light::Light;
pub(crate) use material::Material;
pub(crate) use sphere::Sphere;

use glam::Vec3;

use crate::utils::Rng;
use color::srgb_to_linear;
use environment::Environment;

/// Index into the scene's material list. Only `Scene` can create one,
/// so every `MaterialId` refers to a material that exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MaterialId(u32);

impl MaterialId {
    fn index(self) -> u32 {
        self.0
    }
}

pub(crate) struct Scene {
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

    pub(crate) fn demo() -> Self {
        let mut scene = Self::new();

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

    pub(crate) fn spheres(&self) -> &[Sphere] {
        &self.spheres
    }

    pub(crate) fn materials(&self) -> &[Material] {
        &self.materials
    }

    pub(crate) fn lights(&self) -> &[Light] {
        &self.lights
    }

    pub(crate) fn environment(&self) -> &Environment {
        &self.environment
    }

    fn add_material(&mut self, material: Material) -> MaterialId {
        let id = u32::try_from(self.materials.len()).expect("Material array length overflows u32");
        self.materials.push(material);
        MaterialId(id)
    }

    fn add_sphere(&mut self, center: Vec3, radius: f32, material_id: MaterialId) {
        self.spheres.push(Sphere::new(center, radius, material_id));
    }

    fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub(crate) fn add_random_sphere(&mut self, rng: &mut Rng) {
        if self.materials.is_empty() {
            return;
        }

        let material_count =
            u32::try_from(self.materials.len()).expect("Material array length overflows u32");

        let center = rng.next_vec3((Vec3::new(-5.0, 0.0, -5.0), Vec3::new(5.0, 10.0, 5.0)));
        let radius = rng.next_f32_range(0.5, 2.0);
        let material_id = rng.next_u32_range(0, material_count);
        self.add_sphere(center, radius, MaterialId(material_id));
    }

    pub(crate) fn remove_last_sphere(&mut self) {
        self.spheres.pop();
    }
}
