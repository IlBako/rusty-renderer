use wgpu::{
    BindGroup, BindGroupLayout, Buffer, CommandEncoder, ComputePipeline, Device, Queue, Texture,
    TextureView,
};

use super::{
    gpu_types::{GpuCamera, GpuLight, GpuMaterial, GpuSceneSettings, GpuSphere},
    storage_buffer::StorageBuffer,
};
use crate::scene::{Camera, Scene};

// Size set in the wgsl shader file
const WORKGROUP_SIZE: u32 = 8;

// Initial room in the sphere buffer, in spheres. The buffer grows when a scene needs more.
const INITIAL_SPHERE_CAPACITY: u64 = 16;

// Initial room in the material buffer, in materials. The buffer grows when a scene needs more.
const INITIAL_MATERIAL_CAPACITY: u64 = 4;

// Initial room in the light buffer, in lights. The buffer grows when a scene needs more.
const INITIAL_LIGHT_CAPACITY: u64 = 4;

pub(crate) struct Tracer {
    compute_pipeline: ComputePipeline,
    frame_bind_group: BindGroup,
    scene_bind_group: BindGroup,
    camera_buffer: Buffer,
    sphere_buffer: StorageBuffer,
    material_buffer: StorageBuffer,
    light_buffer: StorageBuffer,
    settings_buffer: Buffer,
    frame_bind_group_layout: BindGroupLayout,
    scene_bind_group_layout: BindGroupLayout,
}

impl Tracer {
    pub(crate) fn new(device: &Device, view: &TextureView) -> Self {
        let compute_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/compute_shader.wgsl"));

        let frame_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Frame Bind Group Layout"),
                entries: &[
                    // @group(0) @binding(0)
                    // var output_texture: texture_storage_2d<rgba8unorm, write>;
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        count: None,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    // @group(0) @binding(1)
                    // var<uniform> camera: Camera;
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                ],
            });

        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Scene Bind Group Layout"),
                entries: &[
                    // @group(1) @binding(0)
                    // var<storage, read> scene: array<Sphere>;
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    // @group(1) @binding(1)
                    // var<storage, read> materials: array<Material>;
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    // @group(1) @binding(2)
                    // var<storage, read> lights: array<Light>;
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    // @group(1) @binding(3)
                    // var<uniform> sceneSettings: GpuSceneSettings;
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            immediate_size: 0,
            bind_group_layouts: &[
                Some(&frame_bind_group_layout),
                Some(&scene_bind_group_layout),
            ],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: size_of::<GpuCamera>() as u64,
        });

        let frame_bind_group =
            Self::build_frame_bind_group(device, view, &camera_buffer, &frame_bind_group_layout);

        let sphere_buffer = StorageBuffer::new(
            device,
            INITIAL_SPHERE_CAPACITY * size_of::<GpuSphere>() as u64,
            "Sphere Buffer",
        );

        let material_buffer = StorageBuffer::new(
            device,
            INITIAL_MATERIAL_CAPACITY * size_of::<GpuMaterial>() as u64,
            "Material Buffer",
        );

        let light_buffer = StorageBuffer::new(
            device,
            INITIAL_LIGHT_CAPACITY * size_of::<GpuLight>() as u64,
            "Light Buffer",
        );

        let settings_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene Settings Buffer"),
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: size_of::<GpuSceneSettings>() as u64,
        });

        let scene_bind_group = Self::build_scene_bind_group(
            device,
            &sphere_buffer,
            &material_buffer,
            &light_buffer,
            &settings_buffer,
            &scene_bind_group_layout,
        );

        Self {
            compute_pipeline,
            frame_bind_group,
            scene_bind_group,
            camera_buffer,
            sphere_buffer,
            material_buffer,
            light_buffer,
            settings_buffer,
            frame_bind_group_layout,
            scene_bind_group_layout,
        }
    }

    pub(crate) fn create_compute_pass(
        &self,
        encoder: &mut CommandEncoder,
        output_texture: &Texture,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            ..Default::default()
        });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.frame_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.scene_bind_group, &[]);
        compute_pass.dispatch_workgroups(
            output_texture.width().div_ceil(WORKGROUP_SIZE),
            output_texture.height().div_ceil(WORKGROUP_SIZE),
            1,
        );
    }

    pub(crate) fn upload_camera(&self, queue: &Queue, camera: &Camera) {
        let camera = GpuCamera::from(camera);
        let camera_bytes: &[u8] = bytemuck::bytes_of(&camera);
        queue.write_buffer(&self.camera_buffer, 0, camera_bytes);
    }

    pub(crate) fn upload_scene(&mut self, device: &Device, queue: &Queue, scene: &Scene) {
        let spheres: Vec<GpuSphere> = scene.spheres().iter().map(GpuSphere::from).collect();
        let materials: Vec<GpuMaterial> = scene.materials().iter().map(GpuMaterial::from).collect();
        let lights: Vec<GpuLight> = scene.lights().iter().map(GpuLight::from).collect();
        let scene_settings = GpuSceneSettings::from(scene);

        let rebuild_group = self
            .sphere_buffer
            .write(device, queue, bytemuck::cast_slice(&spheres))
            | self
                .material_buffer
                .write(device, queue, bytemuck::cast_slice(&materials))
            | self
                .light_buffer
                .write(device, queue, bytemuck::cast_slice(&lights));

        let scene_settings_bytes: &[u8] = bytemuck::bytes_of(&scene_settings);
        queue.write_buffer(&self.settings_buffer, 0, scene_settings_bytes);

        if rebuild_group {
            self.rebuild_scene_bind_group(device);
        }
    }

    fn build_frame_bind_group(
        device: &Device,
        view: &TextureView,
        camera_buffer: &Buffer,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Frame Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera_buffer.as_entire_binding(),
                },
            ],
        })
    }

    fn build_scene_bind_group(
        device: &Device,
        sphere_buffer: &StorageBuffer,
        material_buffer: &StorageBuffer,
        light_buffer: &StorageBuffer,
        settings_buffer: &Buffer,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Scene Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sphere_buffer.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: material_buffer.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: light_buffer.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: settings_buffer.as_entire_binding(),
                },
            ],
        })
    }

    pub(crate) fn rebuild_frame_bind_group(&mut self, device: &Device, view: &TextureView) {
        self.frame_bind_group = Self::build_frame_bind_group(
            device,
            view,
            &self.camera_buffer,
            &self.frame_bind_group_layout,
        );
    }

    fn rebuild_scene_bind_group(&mut self, device: &Device) {
        self.scene_bind_group = Self::build_scene_bind_group(
            device,
            &self.sphere_buffer,
            &self.material_buffer,
            &self.light_buffer,
            &self.settings_buffer,
            &self.scene_bind_group_layout,
        );
    }
}
