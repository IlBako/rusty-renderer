use std::{sync::Arc, time::Instant};

use wgpu::{
    BindGroup, BindGroupLayout, ComputePipeline, Device, Instance, Queue, RenderPipeline, Surface,
    SurfaceConfiguration, Texture, TextureView,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window,
};

pub struct State {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    surface_config: SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: RenderPipeline,
    render_bind_group: BindGroup,
    output_texture: Texture,
    output_view: TextureView,
    compute_pipeline: ComputePipeline,
    compute_bind_group: BindGroup,

    // FPS Counter data
    fps_frame_count: u32,
    fps_last_update: Instant,
}

pub struct App {
    state: Option<State>,
    instance: Instance,
}

impl State {
    pub fn new(window: Arc<Window>, instance: &Instance) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let surface = instance.create_surface(window.clone())?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            alpha_mode: surface_caps.alpha_modes[0],
            color_space: wgpu::SurfaceColorSpace::Auto,
            desired_maximum_frame_latency: 2,
            format: surface_format,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: vec![],
            width: size.width,
        };

        let render_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/display_shader.wgsl"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
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

        let (output_texture, output_view) = get_texture(&device, (size.width, size.height));

        let compute_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/compute_shader.wgsl"));

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("gradient"),
            layout: None,
            module: &compute_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let compute_bind_group = build_bind_group(
            &device,
            Some("Compute Bind Group"),
            &compute_pipeline.get_bind_group_layout(0),
            &output_view,
        );
        let render_bind_group = build_bind_group(
            &device,
            Some("Render Bind Group"),
            &render_pipeline.get_bind_group_layout(0),
            &output_view,
        );

        Ok(Self {
            window,
            surface,
            device,
            queue,
            surface_config,
            is_surface_configured: false,
            render_pipeline,
            render_bind_group,
            output_texture,
            output_view,
            compute_pipeline,
            compute_bind_group,
            fps_frame_count: 0,
            fps_last_update: Instant::now(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            self.is_surface_configured = true;
            (self.output_texture, self.output_view) = get_texture(&self.device, (width, height));
            self.compute_bind_group = build_bind_group(
                &self.device,
                Some("Compute Bind Group"),
                &self.compute_pipeline.get_bind_group_layout(0),
                &self.output_view,
            );
            self.render_bind_group = build_bind_group(
                &self.device,
                Some("Render Bind Group"),
                &self.render_pipeline.get_bind_group_layout(0),
                &self.output_view,
            );
            self.window.request_redraw();
        }
    }

    fn render(&mut self) -> anyhow::Result<()> {
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(());
        }
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                // Skip the frame
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                log::warn!("Validation error!");
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.surface_config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                // Instead of recreating everything, for now just return with an error
                anyhow::bail!("Lost Surface!")
            }
        };

        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        {
            // Size set in the wgsl shader file
            const WORKGROUP_SIZE: u32 = 8;
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                ..Default::default()
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(
                self.output_texture.width().div_ceil(WORKGROUP_SIZE),
                self.output_texture.height().div_ceil(WORKGROUP_SIZE),
                1,
            );
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
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
        self.queue.submit([encoder.finish()]);
        self.queue.present(output);

        self.fps_frame_count += 1;
        let elapsed = self.fps_last_update.elapsed();
        if elapsed.as_secs_f64() >= 1.0 {
            let fps = self.fps_frame_count as f64 / elapsed.as_secs_f64();
            self.window
                .set_title(&format!("Rusty Renderer | {fps:.1} FPS"));
            self.fps_frame_count = 0;
            self.fps_last_update = Instant::now();
        }

        Ok(())
    }
}

impl App {
    pub fn new(instance: Instance) -> Self {
        Self {
            state: None,
            instance,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );
            match State::new(window, &self.instance) {
                Ok(state) => {
                    self.state = Some(state);
                }
                Err(e) => {
                    log::error!("{e}");
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        if state.window.id() != window_id {
            return;
        }
        match event {
            WindowEvent::RedrawRequested => match state.render() {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{e}");
                    event_loop.exit();
                }
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                state.resize(new_size.width, new_size.height);
            }
            _ => (),
        }
    }
}

fn get_texture(device: &Device, size: (u32, u32)) -> (Texture, TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        format: wgpu::TextureFormat::Rgba8Unorm,
        view_formats: &[],
        label: Some("Texture"),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    (texture, view)
}

fn build_bind_group(
    device: &Device,
    label: Option<&str>,
    layout: &BindGroupLayout,
    view: &TextureView,
) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label,
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(view),
        }],
    })
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut inst_desc = wgpu::InstanceDescriptor::new_with_display_handle(Box::new(
        event_loop.owned_display_handle(),
    ));
    inst_desc.backends = wgpu::Backends::DX12;
    let instance = Instance::new(inst_desc);

    let mut app = App::new(instance);
    event_loop.run_app(&mut app).expect("event loop error");
}
