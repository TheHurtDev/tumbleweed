#[derive(Debug, Clone)]
pub struct SurfaceInfo {
    pub width: u32,
    pub height: u32,
}

impl SurfaceInfo {
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

impl Default for SurfaceInfo {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
        }
    }
}
