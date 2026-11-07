
fn main() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() })).expect("no compatible adapter");
    // println!("Adapter: {:#?}", adapter.get_info());

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).expect("device request failed");

    // Data on the CPU
    let input: [f32; 10] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let bytes: &[u8] = bytemuck::cast_slice(&input);
    let size = bytes.len() as u64;
    
    // GPU buffers
    let storage = device.create_buffer(&wgpu::BufferDescriptor{
        label: Some("storage"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor{
        label: Some("readback"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    queue.write_buffer(&storage, 0, bytes);

    // Compile the shader
    let shader = device.create_shader_module(wgpu::include_wgsl!("double.wgsl"));
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor{
        label: Some("double"),
        layout: None, // Derive the layout from the shader
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
        label: Some("double"), 
        layout: &pipeline.get_bind_group_layout(0),     
        entries: &[wgpu::BindGroupEntry{
            binding: 0,
            resource: storage.as_entire_binding(),
        }] 
    });

    // Record work -> nothing yet running on the gpu
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((input.len() as u32).div_ceil(4), 1, 1);
    }
    encoder.copy_buffer_to_buffer(&storage, 0, &readback, 0, size);

    // Hand work to the GPU
    queue.submit([encoder.finish()]);

    // Wait, then read results
    let slice = readback.slice(..);
    slice.map_async(wgpu::MapMode::Read, |result| result.unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

    let mapped = slice.get_mapped_range().unwrap();
    let out: &[u8] = &mapped;
    let output: &[f32] = bytemuck::cast_slice(&mapped);

    println!("input:  {input:?}");
    println!("in:     {bytes:?}");
    println!("out:    {out:?}");
    println!("output: {output:?}");
}
