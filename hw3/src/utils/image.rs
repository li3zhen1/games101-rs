use super::render::TextureConvertible;

pub fn save_image<T: TextureConvertible>(texture: &T, path: &str) {

    let data = texture.dump_u8norm();
    
    image::save_buffer(path, &data, texture.width() as _, texture.height() as _, image::ColorType::Rgba8).unwrap()

}

pub fn save_image_from_u8array(data: &[u8], width: u32, height: u32, path: &str) {
    image::save_buffer(path, data, width, height, image::ColorType::Rgba8).unwrap()
}
