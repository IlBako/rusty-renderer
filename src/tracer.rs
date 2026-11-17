use wgpu::{
    BindGroup, Buffer, CommandEncoder, ComputePipeline, Device, PipelineCompilationOptions, Queue,
    Texture, TextureView,
};

use crate::camera::CameraUniform;

// Size set in the wgsl shader file
const WORKGROUP_SIZE: u32 = 8;

pub(crate) struct Tracer {
    compute_pipeline: ComputePipeline,
    compute_bind_group: BindGroup,
    camera_buffer: Buffer,
}

impl Tracer {
    pub(crate) fn new(device: &Device, view: &TextureView) -> Self {
        let compute_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/compute_shader.wgsl"));

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: None,
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

        let compute_bind_group =
            Self::build_bind_group(&compute_pipeline, device, view, &camera_buffer);

        Self {
            compute_pipeline,
            compute_bind_group,
            camera_buffer,
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
        compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
        compute_pass.dispatch_workgroups(
            output_texture.width().div_ceil(WORKGROUP_SIZE),
            output_texture.height().div_ceil(WORKGROUP_SIZE),
            1,
        );
    }

    pub(crate) fn upload(&self, queue: &Queue, camera: CameraUniform) {
        let camera_bytes: &[u8] = bytemuck::bytes_of(&camera);
        queue.write_buffer(&self.camera_buffer, 0, camera_bytes);
    }

    fn build_bind_group(
        compute_pipeline: &ComputePipeline,
        device: &Device,
        view: &TextureView,
        camera_buffer: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &compute_pipeline.get_bind_group_layout(0),
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

    pub(crate) fn rebuild_bind_group(&mut self, device: &Device, view: &TextureView) {
        self.compute_bind_group =
            Self::build_bind_group(&self.compute_pipeline, device, view, &self.camera_buffer);
    }
}
