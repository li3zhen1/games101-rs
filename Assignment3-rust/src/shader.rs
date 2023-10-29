use glam::*;

use crate::{texture::Texture, rasterizer::PosBufId};

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

pub struct Light {
    pub position: Vec3,
    pub intensity: Vec3,
}


pub type VertexShaderFn = fn(payload: VertexShaderPayload) -> Vec3;

pub type FragmentShaderFn = fn(payload: FragmentShaderPayload) -> Vec4;


pub fn vertex_shader(payload: VertexShaderPayload) -> Vec3 {
    payload.position
}

pub fn texture_fragment_shader(payload: FragmentShaderPayload) -> Vec4 {
    let texture_color = payload.texture.get_color(payload.tex_coords.x, payload.tex_coords.y);
    
    let ka = vec3(0.005, 0.005, 0.005);
    let kd = texture_color / 255.;
    let ks = vec3(0.7937, 0.7937, 0.7937);

    let l1 = Light {
        position: vec3(20., 20., 20.),
        intensity: vec3(500., 500., 500.),
    };

    let l2 = Light {
        position: vec3(-20., 20., 0.),
        intensity: vec3(500., 500., 500.),
    };

    let lights = vec![l1, l2];

    let amb_light_intensity = vec3(10., 10., 10.);
    let eye_pos = vec3(0., 0., 10.);

    let p = 150f32;

    let color = texture_color;
    let point = payload.view_pos;
    let normal = payload.normal;

    let result_color = vec3(0., 0., 0.);
    
    for light in lights {

    }


    result_color.extend(1.)

}


pub fn phong_fragment_shader(payload: FragmentShaderPayload) -> Vec4 {
    let ka = vec3(0.005, 0.005, 0.005);
    let kd = payload.color;
    let ks = vec3(0.7937, 0.7937, 0.7937);

    let l1 = Light {
        position: vec3(20., 20., 20.),
        intensity: vec3(500., 500., 500.),
    };

    let l2 = Light {
        position: vec3(-20., 20., 0.),
        intensity: vec3(500., 500., 500.),
    };

    let lights = vec![l1, l2];

    let amb_light_intensity = vec3(10., 10., 10.);
    let eye_pos = vec3(0., 0., 10.);

    let p = 150f32;

    let color = payload.color;
    let point = payload.view_pos;
    let normal = payload.normal;

    let result_color = vec3(0., 0., 0.);
    
    for light in lights {

    }


    result_color.extend(1.)
}

pub fn displacement_fragment_shader(payload: FragmentShaderPayload) -> Vec4 {
    let ka = vec3(0.005, 0.005, 0.005);
    let kd = payload.color;
    let ks = vec3(0.7937, 0.7937, 0.7937);

    let l1 = Light {
        position: vec3(20., 20., 20.),
        intensity: vec3(500., 500., 500.),
    };

    let l2 = Light {
        position: vec3(-20., 20., 0.),
        intensity: vec3(500., 500., 500.),
    };

    let lights = vec![l1, l2];

    let amb_light_intensity = vec3(10., 10., 10.);
    let eye_pos = vec3(0., 0., 10.);

    let p = 150f32;

    let color = payload.color;
    let point = payload.view_pos;
    let normal = payload.normal;

    let kh = 0.2f32; 
    let kn = 0.1f32;

    let result_color = vec3(0., 0., 0.);
    
    for light in lights {

    }


    result_color.extend(1.)
}




pub fn bump_fragment_shader(payload: FragmentShaderPayload) -> Vec4 {
    let ka = vec3(0.005, 0.005, 0.005);
    let kd = payload.color;
    let ks = vec3(0.7937, 0.7937, 0.7937);

    let l1 = Light {
        position: vec3(20., 20., 20.),
        intensity: vec3(500., 500., 500.),
    };

    let l2 = Light {
        position: vec3(-20., 20., 0.),
        intensity: vec3(500., 500., 500.),
    };

    let lights = vec![l1, l2];

    let amb_light_intensity = vec3(10., 10., 10.);
    let eye_pos = vec3(0., 0., 10.);

    let p = 150f32;

    let color = payload.color;
    let point = payload.view_pos;
    let normal = payload.normal;

    let kh = 0.2f32; 
    let kn = 0.1f32;

        // TODO: Implement bump mapping here
    // Let n = normal = (x, y, z)
    // Vector t = (x*y/sqrt(x*x+z*z),sqrt(x*x+z*z),z*y/sqrt(x*x+z*z))
    // Vector b = n cross product t
    // Matrix TBN = [t b n]
    // dU = kh * kn * (h(u+1/w,v)-h(u,v))
    // dV = kh * kn * (h(u,v+1/h)-h(u,v))
    // Vector ln = (-dU, -dV, 1)
    // Normal n = normalize(TBN * ln)

    let result_color = normal;


    result_color.extend(1.)
}