use std::ops::{Range};

use glam::*;

pub struct Triangle {
    pub v: [Vec3; 3],
    pub color: [Vec4; 3],
    pub tex_coords: [Vec2; 3],
    pub normal: [Vec3; 3],
}

pub struct Rect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    // pub width: f32,
    // pub height: f32,
}

macro_rules! assert_color_is_rgba32float {
    ($color: expr) => {
        assert!($color[0] >= 0.0 && $color[0] <= 1.0);
        assert!($color[1] >= 0.0 && $color[1] <= 1.0);
        assert!($color[2] >= 0.0 && $color[2] <= 1.0);
        assert!($color[3] >= 0.0 && $color[3] <= 1.0);
    };
}

impl Triangle {
    pub fn zeros() -> Self {
        Self {
            v: [
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
            ],
            color: [
                vec4(0.0, 0.0, 0.0, 0.0),
                vec4(0.0, 0.0, 0.0, 0.0),
                vec4(0.0, 0.0, 0.0, 0.0),
            ],
            tex_coords: [vec2(0.0, 0.0), vec2(0.0, 0.0), vec2(0.0, 0.0)],
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

    pub fn set_color(&mut self, index: usize, color: Vec4) {
        assert_color_is_rgba32float!(color);
        self.color[index] = color;
    }

    pub fn set_color_rgba(&mut self, index: usize, r: f32, g: f32, b: f32, a: f32) {
        let color = vec4(r, g, b, a) / 255.0;
        assert_color_is_rgba32float!(color);
        self.color[index] = color;
    }

    pub fn set_color_vec(&mut self, index: usize, color: Vec4) {
        // let color = vec4(r, g, b, 255.0) / 255.0;
        // let color = color;
        assert_color_is_rgba32float!(color);
        self.color[index] = color;
    }

    pub fn set_color_rgb(&mut self, index: usize, r: f32, g: f32, b: f32) {
        let color = vec4(r, g, b, 255.0) / 255.0;
        assert_color_is_rgba32float!(color);
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

    pub fn bounding_box(&self) -> Rect {
        let mut x_min = std::f32::MAX;
        let mut y_min = std::f32::MAX;
        let mut x_max = std::f32::MIN;
        let mut y_max = std::f32::MIN;

        for v in self.v.iter() {
            if v.x < x_min {
                x_min = v.x;
            }
            if v.y < y_min {
                y_min = v.y;
            }
            if v.x > x_max {
                x_max = v.x;
            }
            if v.y > y_max {
                y_max = v.y;
            }
        }

        Rect {
            x0: x_min,
            y0: y_min,
            x1: x_max,
            y1: y_max,
        }
    }

    // //interpolated depth value
    // pub fn get_depth(&self, x: usize, y: usize) -> f32 {
    //     let v0 = vec3(self.v[0].x, self.v[0].y, self.v[0].z);
    //     let v1 = vec3(self.v[1].x, self.v[1].y, self.v[1].z);
    //     let v2 = vec3(self.v[2].x, self.v[2].y, self.v[2].z);

    //     let p = vec3(x as f32, y as f32, 0.0);

    //     let v0v1 = v1 - v0;
    //     let v0v2 = v2 - v0;
    //     let n = v0v1.cross(v0v2);

    //     let d = n.dot(v0);
    //     let z = (d - n.dot(p)) / n.z;

    //     z
    // }

    // pub fn get_color(&self, x: usize, y: usize) -> Vec3 {

    // }
}

impl Rect {
    fn x_start(&self) -> usize {
        self.x0.floor() as _
    }

    fn y_start(&self) -> usize {
        self.y0.floor() as _
    }

    fn x_end(&self) -> usize {
        self.x1.floor() as _
    }

    fn y_end(&self) -> usize {
        self.y1.ceil() as _
    }

    pub fn x_range(&self) -> Range<usize> {
        self.x_start()..(self.x_end() + 1)
    }

    pub fn y_range(&self) -> Range<usize> {
        self.y_start()..(self.y_end() + 1)
    }

}
