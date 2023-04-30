use glam::{vec4, mat4, Mat4, Vec3};

pub fn get_view_matrix(eye_pos: Vec3) -> Mat4 {
    mat4(
        vec4(1.0, 0.0, 0.0, -eye_pos[0]),
        vec4(0.0, 1.0, 0.0, -eye_pos[1]),
        vec4(0.0, 0.0, 1.0, -eye_pos[2]),
        vec4(0.0, 0.0, 0.0, 1.0),
    ) * Mat4::IDENTITY
}

pub fn get_model_matrix(rotation_angle: f32) -> Mat4 {
    let model = Mat4::IDENTITY;
    model
}

pub fn get_projection_matrix(eye_fov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Mat4 {
    let projection = Mat4::IDENTITY;
    projection
}