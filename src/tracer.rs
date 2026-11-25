use wgpu::{
    BindGroup, BindGroupLayout, Buffer, CommandEncoder, ComputePipeline, Device,
    PipelineCompilationOptions, Queue, Texture, TextureView,
};

use crate::{
    camera::CameraUniform,
    scene::{GpuSphere, Scene},
};

// Size set in the wgsl shader file
const WORKGROUP_SIZE: u32 = 8;

// Initial room in the sphere buffer, in spheres. The buffer grows when a scene needs more.
const INITIAL_SPHERE_CAPACITY: u64 = 16;

pub(crate) struct Tracer {
    compute_pipeline: ComputePipeline,
    frame_bind_group: BindGroup,
    scene_bind_group: BindGroup,
    camera_buffer: Buffer,
    sphere_buffer: Buffer,
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
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: size_of::<CameraUniform>() as u64,
        });

        let frame_bind_group =
            Self::build_frame_bind_group(device, view, &camera_buffer, &frame_bind_group_layout);

        // Created empty; the scene is written with `upload_scene`
        let sphere_buffer = Self::create_sphere_buffer(
            device,
            INITIAL_SPHERE_CAPACITY * size_of::<GpuSphere>() as u64,
        );

        let scene_bind_group =
            Self::build_scene_bind_group(device, &sphere_buffer, &scene_bind_group_layout);

        Self {
            compute_pipeline,
            frame_bind_group,
            scene_bind_group,
            camera_buffer,
            sphere_buffer,
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

    pub(crate) fn upload_camera(&self, queue: &Queue, camera: CameraUniform) {
        let camera_bytes: &[u8] = bytemuck::bytes_of(&camera);
        queue.write_buffer(&self.camera_buffer, 0, camera_bytes);
    }

    pub(crate) fn upload_scene(&mut self, device: &Device, queue: &Queue, scene: &Scene) {
        let spheres = scene.get_gpu_spheres();
        let bytes: &[u8] = bytemuck::cast_slice(&spheres);
        let needed = bytes.len() as u64;

        // Grow with spare room, so adding spheres does not recreate the buffer every time.
        // A new buffer needs a new bind group: the old one would keep the old buffer alive.
        if self.sphere_buffer.size() < needed {
            let size = needed.max(self.sphere_buffer.size() * 2);
            self.sphere_buffer = Self::create_sphere_buffer(device, size);
            self.rebuild_scene_bind_group(device);
        }

        queue.write_buffer(&self.sphere_buffer, 0, bytes);
    }

    // Never created with size 0: a runtime-sized array binding must hold at least one element.
    fn create_sphere_buffer(device: &Device, size: u64) -> Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sphere Buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            size,
        })
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
        sphere_buffer: &Buffer,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Scene Bind Group"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sphere_buffer.as_entire_binding(),
            }],
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
            &self.scene_bind_group_layout,
        );
    }
}
