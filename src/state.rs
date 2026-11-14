use std::sync::Arc;

use wgpu::{Instance, Texture, TextureView};
use winit::window::Window;

use crate::{display, fps::FpsCounter, gpu, tracer, utils::get_texture};

pub struct State {
    pub window: Arc<Window>,
    gpu: gpu::Gpu,
    display: display::Display,
    tracer: tracer::Tracer,
    output_texture: Texture,
    output_view: TextureView,
    fps: FpsCounter,
}

impl State {
    pub fn new(window: Arc<Window>, instance: &Instance) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let gpu = gpu::Gpu::new(window.clone(), instance)?;

        let (output_texture, output_view) = get_texture(&gpu.device, (size.width, size.height));

        let display = display::Display::new(&gpu.device, gpu.surface_format(), &output_view);

        let tracer = tracer::Tracer::new(&gpu.device, &output_view);

        Ok(Self {
            window,
            gpu,
            display,
            tracer,
            output_texture,
            output_view,
            fps: FpsCounter::new(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.gpu.resize(width, height);

        (self.output_texture, self.output_view) = get_texture(&self.gpu.device, (width, height));

        self.display
            .rebuild_bind_group(&self.gpu.device, &self.output_view);
        self.tracer
            .rebuild_bind_group(&self.gpu.device, &self.output_view);

        self.window.request_redraw();
    }

    pub(crate) fn render(&mut self) -> anyhow::Result<()> {
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(());
        }
        self.window.request_redraw();

        let Some(output) = self.gpu.next_frame()? else {
            return Ok(());
        };

        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        self.tracer
            .create_compute_pass(&mut encoder, &self.output_texture);

        self.display.create_render_pass(&mut encoder, &surface_view);

        self.gpu.queue.submit([encoder.finish()]);
        self.gpu.queue.present(output);

        if let Some(fps) = self.fps.frame() {
            self.window
                .set_title(&format!("Rusty Renderer | {fps:.1} FPS"));
        }

        Ok(())
    }
}
