use wgpu::{Buffer, Device, Queue};

/// A storage buffer that grows when the data written to it no longer fits.
pub(super) struct StorageBuffer {
    buffer: Buffer,
    label: &'static str,
}

impl StorageBuffer {
    pub(super) fn new(device: &Device, initial_capacity: u64, label: &'static str) -> Self {
        // Ensure non-zero, 4-byte aligned initial capacity so doubling (`size * 2`) always grows.
        let size = initial_capacity.max(4);
        Self {
            buffer: Self::create_buffer(device, size, label),
            label,
        }
    }

    /// Grows the buffer if `bytes` does not fit, writes the data, and returns
    /// `true` if a new buffer was allocated (requiring a bind group rebuild).
    pub(super) fn write(&mut self, device: &Device, queue: &Queue, bytes: &[u8]) -> bool {
        let needed = bytes.len() as u64;
        let mut reallocated = false;

        if self.buffer.size() < needed {
            let size = needed.max(self.buffer.size() * 2);
            self.buffer = Self::create_buffer(device, size, self.label);
            reallocated = true;
        }

        queue.write_buffer(&self.buffer, 0, bytes);
        reallocated
    }

    pub(super) fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    fn create_buffer(device: &Device, size: u64, label: &'static str) -> Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}
