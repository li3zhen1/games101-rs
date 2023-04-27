mod triangle;
mod rasterizer;

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use glam::{Mat4, mat4, uvec3, vec3, Vec3, vec4};
use triangle::Triangle;
use rasterizer::Rasterizer;
use crate::rasterizer::PrimitiveKind;


fn get_view_matrix(eye_pos: Vec3) -> Mat4 {
    mat4(
        vec4(1.0, 0.0, 0.0, -eye_pos[0]),
        vec4(0.0, 1.0, 0.0, -eye_pos[1]),
        vec4(0.0, 0.0, 1.0, -eye_pos[2]),
        vec4(0.0, 0.0, 0.0, 1.0),
    ) * Mat4::IDENTITY
}

fn get_model_matrix(rotation_angle: f32) -> Mat4{
    let model = Mat4::IDENTITY;
    model
}

fn get_projection_matrix(eye_fov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Mat4{
    let projection = Mat4::IDENTITY;
    projection
}



fn main() {
    let mut r = Rasterizer::new(700, 700);

    let mut t = Triangle::new();


    let pos_id = r.load_positions(vec![
        vec3(2.0, 0.0, -2.0),
        vec3(0.0, 2.0, -2.0),
        vec3(-2.0, 0.0, -2.0),
    ]);

    let ind_id = r.load_indices(vec![uvec3(0, 1, 2)]);


    let eye_pos = vec3(0.0, 0.0, 5.0);
    let angle = 0f32;

    r.set_model(get_model_matrix(angle));
    r.set_view(get_view_matrix(eye_pos));
    r.set_projection(get_projection_matrix(45.0, 1.0, 0.1, 50.0));

    r.draw(&pos_id, &ind_id, PrimitiveKind::Triangle);


    let path = Path::new(r"image.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 700, 700); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));     // 1.0 / 2.2, unscaled, but rounded
    let source_chromaticities = png::SourceChromaticities::new(     // Using unscaled instantiation here
                                                                    (0.31270, 0.32900),
                                                                    (0.64000, 0.33000),
                                                                    (0.30000, 0.60000),
                                                                    (0.15000, 0.06000)
    );
    encoder.set_source_chromaticities(source_chromaticities);
    let mut writer = encoder.write_header().unwrap();

    let data = r.dump_pixels();
    writer.write_image_data(&data).unwrap(); // Save


}