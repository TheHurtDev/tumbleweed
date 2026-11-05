use anyhow::anyhow;
use hawthorn::math::{Vec3, vec3};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
}

struct MeshData {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

#[derive(Clone)]
pub struct Mesh(Arc<MeshData>);

impl Mesh {
    pub fn try_new(
        vertices: impl Into<Vec<Vertex>>,
        indices: impl Into<Vec<u16>>,
    ) -> anyhow::Result<Self> {
        let vertices = vertices.into();
        let indices = indices.into();

        if indices.is_empty() {
            return Err(anyhow!("indices cannot be empty"));
        }
        if indices.len() % 3 != 0 {
            return Err(anyhow!("mesh is not triangulated"));
        }

        if let Some(&i) = indices.iter().find(|&&i| i as usize >= vertices.len()) {
            return Err(anyhow!(
                "index {i:?} is invalid for vertices of length {:?}",
                vertices.len()
            ));
        }

        Ok(Self(Arc::new(MeshData { vertices, indices })))
    }

    pub fn new(vertices: impl Into<Vec<Vertex>>, indices: impl Into<Vec<u16>>) -> Self {
        Self::try_new(vertices, indices).unwrap()
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.0.vertices
    }

    pub fn indices(&self) -> &[u16] {
        &self.0.indices
    }

    pub fn positions(&self) -> impl Iterator<Item = Vec3> {
        self.vertices().iter().map(|v| v.position)
    }

    pub fn normals(&self) -> impl Iterator<Item = Vec3> {
        self.vertices().iter().map(|v| v.normal)
    }
}

impl Mesh {
    pub fn new_flat_shaded_cube() -> Self {
        const VERTICES: [Vertex; 24] = [
            // Front
            Vertex {
                position: vec3(-0.5, -0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
            },
            Vertex {
                position: vec3(-0.5, 0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
            },
            Vertex {
                position: vec3(0.5, 0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
            },
            Vertex {
                position: vec3(0.5, -0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
            },
            // Left
            Vertex {
                position: vec3(-0.5, -0.5, -0.5),
                normal: vec3(-1.0, 0.0, 0.0),
            },
            Vertex {
                position: vec3(-0.5, 0.5, -0.5),
                normal: vec3(-1.0, 0.0, 0.0),
            },
            Vertex {
                position: vec3(-0.5, 0.5, 0.5),
                normal: vec3(-1.0, 0.0, 0.0),
            },
            Vertex {
                position: vec3(-0.5, -0.5, 0.5),
                normal: vec3(-1.0, 0.0, 0.0),
            },
            // Back
            Vertex {
                position: vec3(0.5, -0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
            },
            Vertex {
                position: vec3(0.5, 0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
            },
            Vertex {
                position: vec3(-0.5, 0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
            },
            Vertex {
                position: vec3(-0.5, -0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
            },
            // Right
            Vertex {
                position: vec3(0.5, -0.5, 0.5),
                normal: vec3(1.0, 0.0, 0.0),
            },
            Vertex {
                position: vec3(0.5, 0.5, 0.5),
                normal: vec3(1.0, 0.0, 0.0),
            },
            Vertex {
                position: vec3(0.5, 0.5, -0.5),
                normal: vec3(1.0, 0.0, 0.0),
            },
            Vertex {
                position: vec3(0.5, -0.5, -0.5),
                normal: vec3(1.0, 0.0, 0.0),
            },
            // Top
            Vertex {
                position: vec3(-0.5, 0.5, 0.5),
                normal: vec3(0.0, 1.0, 0.0),
            },
            Vertex {
                position: vec3(-0.5, 0.5, -0.5),
                normal: vec3(0.0, 1.0, 0.0),
            },
            Vertex {
                position: vec3(0.5, 0.5, -0.5),
                normal: vec3(0.0, 1.0, 0.0),
            },
            Vertex {
                position: vec3(0.5, 0.5, 0.5),
                normal: vec3(0.0, 1.0, 0.0),
            },
            // Down
            Vertex {
                position: vec3(-0.5, -0.5, -0.5),
                normal: vec3(0.0, -1.0, 0.0),
            },
            Vertex {
                position: vec3(-0.5, -0.5, 0.5),
                normal: vec3(0.0, -1.0, 0.0),
            },
            Vertex {
                position: vec3(0.5, -0.5, 0.5),
                normal: vec3(0.0, -1.0, 0.0),
            },
            Vertex {
                position: vec3(0.5, -0.5, -0.5),
                normal: vec3(0.0, -1.0, 0.0),
            },
        ];

        const INDICES: [u16; 36] = [
            0, 2, 1, 2, 0, 3, 4, 6, 5, 6, 4, 7, 8, 10, 9, 10, 8, 11, 12, 14, 13, 14, 12, 15, 16,
            18, 17, 18, 16, 19, 20, 22, 21, 22, 20, 23,
        ];

        Self::new(VERTICES.to_vec(), INDICES.to_vec())
    }

    pub fn new_smooth_shaded_cube() -> Self {
        const VERTICES: [Vertex; 8] = [
            Vertex {
                position: vec3(-0.5, -0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
            },
            Vertex {
                position: vec3(-0.5, 0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
            },
            Vertex {
                position: vec3(0.5, 0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
            },
            Vertex {
                position: vec3(0.5, -0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
            },
            Vertex {
                position: vec3(0.5, -0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
            },
            Vertex {
                position: vec3(0.5, 0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
            },
            Vertex {
                position: vec3(-0.5, 0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
            },
            Vertex {
                position: vec3(-0.5, -0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
            },
        ];

        const INDICES: [u16; 36] = [
            0, 3, 2, 0, 2, 1, 4, 7, 6, 4, 6, 5, 0, 1, 6, 0, 6, 7, 3, 4, 5, 3, 5, 2, 1, 2, 5, 1, 5,
            6, 0, 7, 4, 0, 4, 3,
        ];

        Self::new(VERTICES.to_vec(), INDICES.to_vec())
    }

    pub fn new_uv_sphere(level_of_detail: u16) -> Mesh {
        const RADIUS: f32 = 0.5;

        let stack_count = 2 * (level_of_detail + 1);
        let slice_count = 2 * stack_count;

        let mut vertices: Vec<Vertex> = Vec::new();

        for y in 0..=stack_count {
            let theta = (y as f32 / stack_count as f32) * core::f32::consts::PI;
            let (sin_theta, cos_theta) = theta.sin_cos();

            for x in 0..=slice_count {
                let phi = (x as f32 / slice_count as f32) * 2.0 * core::f32::consts::PI;
                let (sin_phi, cos_phi) = phi.sin_cos();

                let normal = vec3(sin_theta * cos_phi, cos_theta, sin_theta * sin_phi);

                vertices.push(Vertex {
                    position: RADIUS * normal,
                    normal,
                });
            }
        }

        let mut indices: Vec<u16> = Vec::new();

        for y in 0..stack_count {
            for x in 0..slice_count {
                let i0 = y * (slice_count + 1) + x;
                let i1 = i0 + 1;
                let i2 = i0 + slice_count + 1;
                let i3 = i2 + 1;

                indices.extend_from_slice(&[i0, i1, i2]);
                indices.extend_from_slice(&[i1, i3, i2]);
            }
        }

        Self::new(vertices, indices)
    }
}
