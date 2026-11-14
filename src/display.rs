use wgpu::{BindGroup, CommandEncoder, Device, RenderPipeline, TextureFormat, TextureView};

use crate::utils::create_bind_group;

pub(crate) struct Display {
    render_pipeline: RenderPipeline,
    render_bind_group: BindGroup,
}

impl Display {
    pub(crate) fn new(device: &Device, format: TextureFormat, view: &TextureView) -> Self {
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/display_shader.wgsl"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                conservative: false,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let render_bind_group = Self::build_bind_group(&render_pipeline, device, view);

        Self {
            render_pipeline,
            render_bind_group,
        }
    }

    pub(crate) fn create_render_pass(
        &self,
        encoder: &mut CommandEncoder,
        surface_view: &TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            multiview_mask: None,
            timestamp_writes: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.render_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn build_bind_group(
        render_pipeline: &RenderPipeline,
        device: &Device,
        view: &TextureView,
    ) -> BindGroup {
        create_bind_group(
            device,
            Some("Render Bind Group"),
            &render_pipeline.get_bind_group_layout(0),
            view,
        )
    }

    pub(crate) fn rebuild_bind_group(&mut self, device: &Device, view: &TextureView) {
        self.render_bind_group = Self::build_bind_group(&self.render_pipeline, device, view);
    }
}
