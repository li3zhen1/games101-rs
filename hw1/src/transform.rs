use glam::*;

pub fn get_view_matrix(eye_pos: Vec3) -> Mat4 {
    mat4(
        vec4(1.0, 0.0, 0.0, -eye_pos[0]),
        vec4(0.0, 1.0, 0.0, -eye_pos[1]),
        vec4(0.0, 0.0, 1.0, -eye_pos[2]),
        vec4(0.0, 0.0, 0.0, 1.0),
    ).transpose()
}

pub fn get_model_matrix(rotation_angle_deg: f32) -> Mat4 {

    get_rotation(vec3(0.,0.,1.), rotation_angle_deg)
    // let angle = rotation_angle_deg.to_radians();
    // mat4(
    //     vec4(angle.cos(), -angle.sin(), 0., 0.),
    //     vec4(angle.sin(), angle.cos(), 0., 0.),
    //     vec4(0., 0., 1., 0.),
    //     vec4(0., 0., 0., 1.),
    // ).transpose()
}

pub fn get_projection_matrix(eye_fov_deg: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Mat4 {
    let top = -((eye_fov_deg/2.).to_radians() * z_near.abs()).tan();
    let right = top * aspect_ratio;

    mat4(
        vec4(z_near / right, 0., 0., 0.),
        vec4(0., z_near / top, 0., 0.),
        vec4(0., 0., (z_near + z_far) / (z_near - z_far), (2. * z_near * z_far) / (z_far - z_near)),
        vec4(0., 0., 1., 0.),
    ).transpose()
}


pub fn get_rotation(axis: Vec3, deg: f32) -> Mat4 {
    let angle = deg.to_radians();

    let u = mat3(
        vec3(0., -axis[2], axis[1]),
        vec3(axis[2], 0., -axis[0]),
        vec3(-axis[1], axis[0], 0.)
    ).transpose();

    let partial = angle.cos() * Mat3::IDENTITY + (1. - angle.cos())*u*u.transpose() + angle.sin()*u;

    Mat4::from_mat3(partial)
}
