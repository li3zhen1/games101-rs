#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TexturedVertex {
    pub position: u64,
    pub texture_coord: u64,
}

impl TexturedVertex {
    pub fn new(
        x: f32,
        y: f32,
        u: f32,
        v: f32,
    ) -> Self {
        unsafe {
            Self {
                position: std::mem::transmute([x, y]),
                texture_coord: std::mem::transmute([u, v])
            }
        }
    }
}

#[repr(C)]
pub enum VertexInput {
    Vertices = 0,
    ViewportSize = 1
}

#[repr(C)]
pub enum TextureIndex {
    BaseColor = 0,
}