use glam::*;

pub struct Triangle {
    v: [Vec3; 3],
    color: [Vec3; 3],
    tex_coords: [Vec2; 3],
    normal: [Vec3; 3],
}

macro_rules! assert_color {
    ($color: expr) => {
        assert!($color[0] >= 0.0 && $color[0] <= 1.0);
        assert!($color[1] >= 0.0 && $color[1] <= 1.0);
        assert!($color[2] >= 0.0 && $color[2] <= 1.0);
    };
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            v: [
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
            ],
            color: [
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
            ],
            tex_coords: [
                vec2(0.0, 0.0),
                vec2(0.0, 0.0),
                vec2(0.0, 0.0),
            ],
            normal: [
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
            ],
        }
    }

    pub fn a(&self) -> Vec3 {
        self.v[0]
    }

    pub fn b(&self) -> Vec3 {
        self.v[1]
    }

    pub fn c(&self) -> Vec3 {
        self.v[2]
    }

    pub fn set_vertex(&mut self, index: usize, v: Vec3) {
        self.v[index] = v;
    }

    pub fn set_color(&mut self, index: usize, color: Vec3) {
        assert_color!(color);
        self.color[index] = color;
    }

    pub fn set_color_rgb(&mut self, index: usize, r: f32, g: f32, b: f32) {
        let color = vec3(r, g, b) / 255.0;
        assert_color!(color);
        self.color[index] = color;
    }

    pub fn set_tex_coords(&mut self, index: usize, s: f32, t: f32) {
        self.tex_coords[index] = Vec2::new(s, t);
    }

    pub fn set_normal(&mut self, index: usize, normal: Vec3) {
        self.normal[index] = normal;
    }

    pub fn to_vec4(&self) -> [Vec4; 3] {
        self.v.map(|v| v.extend(1.0))
    }
}