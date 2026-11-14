use wgpu::{BindGroup, BindGroupLayout, Device, Texture, TextureView};

pub(crate) fn create_bind_group(
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

pub(crate) fn get_texture(device: &Device, size: (u32, u32)) -> (Texture, TextureView) {
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
