use glam::*;
use image::Rgb;

pub struct Texture {
    image_data: image::RgbImage,
}

fn rgb_data_to_vec4(data: &Rgb<u8>) -> Vec3 {
    vec3(
        data[0] as f32 / 255.,
        data[1] as f32 / 255.,
        data[1] as f32 / 255.,
    )
}

fn interpolate(x0: u32, x1: u32, x: f32, y0: Vec3, y1: Vec3) -> Vec3 {
    if x0 == x1 {
        return y0;
    }
    let x0 = x0 as f32;
    let x1 = x1 as f32;
    (y1 - y0) / (x1 - x0) * (x - x0) + y0
}

impl Texture {
    
    pub fn new(path: &str) -> Self {
        let image_data = image::open(path).unwrap().into_rgb8();
        Self { image_data }
    }

    pub fn get_color(&self, x: f32, y: f32) -> Vec3 {
        let x1 = x.floor() as u32;
        let x2 = x.ceil() as u32;
        let y1 = y.floor() as u32;
        let y2 = y.ceil() as u32;

        let points = [(x1, y1), (x1, y2), (x2, y1), (x2, y2)];

        let color = points.map(|p| rgb_data_to_vec4(self.image_data.get_pixel(p.0, p.1)));

        let c0 = interpolate(x1, x2, x, color[0], color[2]);
        let c1 = interpolate(x1, x2, x, color[1], color[3]);
        interpolate(y1, y2, y, c0, c1)
    }

    pub fn get_color_by_tex_coord(&self, tex_coord: Vec2) -> Vec3 {
        let x = tex_coord.x * (self.width() - 1) as f32;
        let y = tex_coord.y * (self.height() - 1) as f32;
        let x1 = x.floor() as u32;
        let x2 = x.ceil() as u32;
        let y1 = y.floor() as u32;
        let y2 = y.ceil() as u32;

        let points = [(x1, y1), (x1, y2), (x2, y1), (x2, y2)];

        let color = points.map(|p| {
            let raw = self.image_data.get_pixel_checked(p.0, p.1);
            match raw {
                Some(color) => rgb_data_to_vec4(color),
                None => {
                    Vec3 { x: 1., y: 0., z: 0. }
                }
            }
        });

        let c0 = interpolate(x1, x2, x, color[0], color[2]);
        let c1 = interpolate(x1, x2, x, color[1], color[3]);
        interpolate(y1, y2, y, c0, c1)
    }

    pub fn width(&self) -> u32 {
        self.image_data.width()
    }

    pub fn height(&self) -> u32 {
        self.image_data.height()
    }
}
