use std::mem;
use wgpu::util::DeviceExt;

use crate::app::{Mesh, Vertex};

pub(in crate::app) struct MeshBuffers {
    pub(in crate::app) vertex_buffer: wgpu::Buffer,
    pub(in crate::app) index_buffer: wgpu::Buffer,
    pub(in crate::app) index_count: u32,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct MeshVertex {
    position: [f32; 3],
    normal: [f32; 3],
}

impl MeshBuffers {
    pub(in crate::app) fn from_mesh(mesh: Mesh, device: &wgpu::Device) -> Self {
        let vertices = mesh
            .vertices()
            .iter()
            .map(MeshVertex::from_vertex)
            .collect::<Vec<_>>();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(mesh.indices()),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices().len() as u32,
        }
    }

    pub(in crate::app) fn standard_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }

    pub(in crate::app) fn positions_only_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

impl MeshVertex {
    fn from_vertex(vertex: &Vertex) -> Self {
        Self {
            position: vertex.position.to_array(),
            normal: vertex.normal.to_array(),
        }
    }
}
