use std::num::NonZeroU64;

#[derive(Debug, Clone)]
pub(in crate::app) struct UniformsBuffer {
    pub(in crate::app) buffer: wgpu::Buffer,
    pub(in crate::app) bind_group: wgpu::BindGroup,

    vertex_binding_size: usize,
    fragment_binding_size: usize,
}

impl UniformsBuffer {
    pub(in crate::app) const SIZE: usize = 10_240;

    pub(in crate::app) fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        vertex_binding_size: usize,
        fragment_binding_size: usize,
    ) -> Self {
        let alignment = device_aligned_offset(vertex_binding_size, device)
            + device_aligned_offset(fragment_binding_size, device);

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms buffer"),
            size: alignment * Self::SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut buffer_bindings: Vec<wgpu::BufferBinding> = Vec::new();

        if vertex_binding_size > 0 {
            buffer_bindings.push(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: NonZeroU64::new(vertex_binding_size as u64),
            });
        }

        if fragment_binding_size > 0 {
            buffer_bindings.push(wgpu::BufferBinding {
                buffer: &buffer,
                offset: device_aligned_offset(vertex_binding_size, device),
                size: NonZeroU64::new(fragment_binding_size as u64),
            });
        }

        let bind_group_entries = buffer_bindings
            .into_iter()
            .enumerate()
            .map(|(i, b)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: wgpu::BindingResource::Buffer(b),
            })
            .collect::<Vec<_>>();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniforms bind group"),
            layout: bind_group_layout,
            entries: &bind_group_entries,
        });

        Self {
            buffer,
            bind_group,
            vertex_binding_size,
            fragment_binding_size,
        }
    }

    pub(in crate::app) fn update(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uniforms: &[&[u8]],
    ) {
        let alignment = self.alignment(device);

        for (i, bytes) in uniforms.iter().enumerate() {
            let offset = i as u64 * alignment;

            queue.write_buffer(&self.buffer, offset, &bytes[..self.vertex_binding_size]);

            queue.write_buffer(
                &self.buffer,
                offset + device_aligned_offset(self.vertex_binding_size, device),
                &bytes[self.vertex_binding_size..],
            );
        }
    }

    pub(in crate::app) fn alignment(&self, device: &wgpu::Device) -> u64 {
        device_aligned_offset(self.vertex_binding_size, device)
            + device_aligned_offset(self.fragment_binding_size, device)
    }
}

#[inline]
fn device_aligned_offset(offset: usize, device: &wgpu::Device) -> u64 {
    let m = device.limits().min_uniform_buffer_offset_alignment as usize;

    (offset.div_ceil(m) * m) as u64
}
