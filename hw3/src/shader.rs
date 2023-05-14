use glam::*;

use crate::texture::Texture;

pub struct FragmentShaderPayload<'a> {
    view_pos: Vec3,
    color: Vec3,
    normal: Vec3,
    tex_coords: Vec2,
    
    texture: &'a Texture
}

pub struct VertexShaderPayload {
    position: Vec3
}