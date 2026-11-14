use wgpu::{
    BindGroup, CommandEncoder, ComputePipeline, Device, PipelineCompilationOptions, Texture,
    TextureView,
};

use crate::utils::create_bind_group;

// Size set in the wgsl shader file
const WORKGROUP_SIZE: u32 = 8;

pub(crate) struct Tracer {
    compute_pipeline: ComputePipeline,
    compute_bind_group: BindGroup,
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

        let compute_bind_group = Self::build_bind_group(&compute_pipeline, device, view);

        Self {
            compute_pipeline,
            compute_bind_group,
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

    fn build_bind_group(
        compute_pipeline: &ComputePipeline,
        device: &Device,
        view: &TextureView,
    ) -> BindGroup {
        create_bind_group(
            device,
            Some("Compute Bind Group"),
            &compute_pipeline.get_bind_group_layout(0),
            view,
        )
    }

    pub(crate) fn rebuild_bind_group(&mut self, device: &Device, view: &TextureView) {
        self.compute_bind_group = Self::build_bind_group(&self.compute_pipeline, device, view);
    }
}
