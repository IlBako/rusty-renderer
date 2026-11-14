use std::sync::Arc;

use wgpu::{Device, Instance, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

pub(crate) struct Gpu {
    surface: Surface<'static>,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    surface_config: SurfaceConfiguration,
}

impl Gpu {
    pub(crate) fn new(window: Arc<Window>, instance: &Instance) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let surface = instance.create_surface(window)?;

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

        let gpu = Self {
            surface,
            device,
            queue,
            surface_config,
        };

        if size.width > 0 && size.height > 0 {
            gpu.configure();
        }

        Ok(gpu)
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.configure();
    }

    pub(crate) fn next_frame(&self) -> anyhow::Result<Option<wgpu::SurfaceTexture>> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => Ok(Some(surface_texture)),
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                // Skip the frame
                Ok(None)
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                log::warn!("Validation error!");
                Ok(None)
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure();
                Ok(None)
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                // Instead of recreating everything, for now just return with an error
                anyhow::bail!("Lost Surface!")
            }
        }
    }

    pub(crate) fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }

    fn configure(&self) {
        self.surface.configure(&self.device, &self.surface_config);
    }
}
