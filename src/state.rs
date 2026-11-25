use std::sync::Arc;

use glam::Vec3;
use wgpu::{Instance, Texture, TextureView};
use winit::window::Window;

use crate::{camera, display, fps::FpsCounter, gpu, scene, tracer, utils::get_texture};

pub struct State {
    pub window: Arc<Window>,
    gpu: gpu::Gpu,
    display: display::Display,
    tracer: tracer::Tracer,
    output_texture: Texture,
    output_view: TextureView,
    camera: camera::Camera,
    scene: scene::Scene,
    fps: FpsCounter,
}

impl State {
    pub fn new(window: Arc<Window>, instance: &Instance) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let gpu = gpu::Gpu::new(window.clone(), instance)?;

        let (output_texture, output_view) = get_texture(&gpu.device, (size.width, size.height));

        let display = display::Display::new(&gpu.device, gpu.surface_format(), &output_view);

        let camera = camera::Camera::new(Vec3::new(0.0, 0.0, -3.0), 5.0, 60.0_f32.to_radians());

        let scene = scene::Scene::demo();

        let mut tracer = tracer::Tracer::new(&gpu.device, &output_view);
        tracer.upload_scene(&gpu.device, &gpu.queue, &scene);

        Ok(Self {
            window,
            gpu,
            display,
            tracer,
            output_texture,
            output_view,
            camera,
            scene,
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
            .rebuild_frame_bind_group(&self.gpu.device, &self.output_view);

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

        self.tracer
            .upload_camera(&self.gpu.queue, self.camera.to_uniform());

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

    pub(crate) fn pan(&mut self, dx: f64, dy: f64) {
        self.camera.pan(dx, dy);
    }

    pub(crate) fn orbit(&mut self, dx: f64, dy: f64) {
        self.camera.orbit(dx, dy);
    }

    pub(crate) fn dolly(&mut self, amount: f32) {
        self.camera.dolly(amount);
    }

    pub(crate) fn reset_camera(&mut self) {
        self.camera.reset_camera();
    }
}
