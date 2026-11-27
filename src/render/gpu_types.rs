//! GPU-side mirrors of the scene types, laid out to match the WGSL structs in
//! `shaders/compute_shader.wgsl`, plus the conversions from the CPU-side types.
//! WGSL aligns `vec3<f32>` to 16 bytes, hence the explicit padding fields.

use bytemuck::{Pod, Zeroable};

use crate::scene::{Camera, Light, Material, Scene, Sphere};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(super) struct GpuCamera {
    position: [f32; 3],
    fov: f32,
    right: [f32; 4],
    up: [f32; 4],
    forward: [f32; 4],
}

const _: () = assert!(size_of::<GpuCamera>() == 64);

impl From<&Camera> for GpuCamera {
    fn from(camera: &Camera) -> Self {
        let (right, up, forward) = camera.axes();

        Self {
            position: camera.position().to_array(),
            fov: camera.fov(),
            right: right.extend(0.0).to_array(),
            up: up.extend(0.0).to_array(),
            forward: forward.extend(0.0).to_array(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(super) struct GpuSphere {
    center: [f32; 3],
    radius: f32,
    material_idx: u32,
    _padding: [f32; 3],
}

const _: () = assert!(size_of::<GpuSphere>() == 32);

impl From<&Sphere> for GpuSphere {
    fn from(sphere: &Sphere) -> Self {
        Self {
            center: sphere.center().to_array(),
            radius: sphere.radius(),
            material_idx: sphere.material_index(),
            _padding: [0.0; 3],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(super) struct GpuMaterial {
    albedo: [f32; 3],
    _padding: f32,
}

const _: () = assert!(size_of::<GpuMaterial>() == 16);

impl From<&Material> for GpuMaterial {
    fn from(material: &Material) -> Self {
        Self {
            albedo: material.albedo().to_array(),
            _padding: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(super) struct GpuLight {
    color: [f32; 3],
    intensity: f32,
    direction: [f32; 3],
    _padding: f32,
}

const _: () = assert!(size_of::<GpuLight>() == 32);

impl From<&Light> for GpuLight {
    fn from(light: &Light) -> Self {
        Self {
            color: light.color().to_array(),
            intensity: light.intensity(),
            direction: light.direction().to_array(),
            _padding: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(super) struct GpuSceneSettings {
    sky_color: [f32; 3],
    ambient: f32,
    sphere_count: u32,
    light_count: u32,
    _padding: [f32; 2],
}

const _: () = assert!(size_of::<GpuSceneSettings>() == 32);

impl From<&Scene> for GpuSceneSettings {
    fn from(scene: &Scene) -> Self {
        let environment = scene.environment();

        Self {
            sky_color: environment.sky_color().to_array(),
            ambient: environment.ambient(),
            sphere_count: u32::try_from(scene.spheres().len())
                .expect("Spheres count overflows u32::MAX"),
            light_count: u32::try_from(scene.lights().len())
                .expect("Lights count overflows u32::MAX"),
            _padding: [0.0; 2],
        }
    }
}
