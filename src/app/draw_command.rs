use hawthorn::math::{Mat4, Vec3};

/// Info needed by the renderer for one draw call.
#[derive(Debug, Clone)]
pub struct DrawCommand {
    pub mesh_name: String,
    pub world_matrix: Mat4,
    pub color: Vec3,
}
