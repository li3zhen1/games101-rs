mod rasterizer;
mod shader_types;
mod transform;
mod triangle;
mod window;

use crate::rasterizer::Rasterizer;
use crate::shader_types::TexturedVertex;
use crate::transform::*;
use crate::triangle::Triangle;
use crate::window::*;
use glam::{uvec3, vec3};
use metal::{Device, MTLPixelFormat, MTLResourceOptions};
use objc::rc::autoreleasepool;
use rasterizer::PrimitiveKind;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

const INITIAL_WINDOW_WIDTH: u32 = 600;
const INITIAL_WINDOW_HEIGHT: u32 = 600;

// CAMetalLayer only accepts the following pixel formats:
// https://developer.apple.com/documentation/quartzcore/cametallayer/1478155-pixelformat
const TEXTURE_PIXEL_FORMAT: MTLPixelFormat = MTLPixelFormat::RGBA32Float;
const PIPE_LINE_PIXEL_FORMAT: MTLPixelFormat = MTLPixelFormat::RGBA8Unorm_sRGB;
const VERTEX_SHADER_NAME: &str = "quad_vertex";
const FRAGMENT_SHADER_NAME: &str = "sampling_shader";
const SHADER_FILE_NAME: &str = env!("SHADER_FILE_NAME");

fn main() {
    let mut r = Rasterizer::new(INITIAL_WINDOW_WIDTH as _, INITIAL_WINDOW_HEIGHT as _);

    let mut t = Triangle::zeros();

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

    let event_loop = EventLoop::new();
    let window = create_window(
        &event_loop,
        INITIAL_WINDOW_WIDTH,
        INITIAL_WINDOW_HEIGHT,
        "HW1_Triangle",
    );

    let device = Device::system_default().unwrap();

    let library = device
        .new_library_with_file(get_metal_lib(
            format!("{SHADER_FILE_NAME}.metallib").as_str(),
        ))
        .unwrap();

    let viewport_size = [window.inner_size().width, window.inner_size().height];

    let vertex_data = get_vertices(viewport_size[0] as f32, viewport_size[1] as f32);

    let vertex_buffer = device.new_buffer_with_data(
        vertex_data.as_ptr() as *const _,
        std::mem::size_of::<[TexturedVertex; 6]>() as _,
        MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeShared,
    );

    let viewport_size_buffer = device.new_buffer(
        std::mem::size_of::<[u32; 2]>() as _,
        MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeShared,
    );

    update_viewport_size(&viewport_size_buffer, viewport_size);

    let texture = create_texture(&r, &device, TEXTURE_PIXEL_FORMAT);

    let layer = get_window_layer(&window, &device, PIPE_LINE_PIXEL_FORMAT);

    let pipeline_state = prepare_pipeline_state(
        &device,
        &library,
        VERTEX_SHADER_NAME,
        FRAGMENT_SHADER_NAME,
        PIPE_LINE_PIXEL_FORMAT,
    );

    let command_queue = device.new_command_queue();

    event_loop.run(move |event, _, control_flow| {
        autoreleasepool(|| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event, .. } => {
                    handle_window_event(event, control_flow, &layer, &viewport_size_buffer);
                }
                Event::MainEventsCleared => window.request_redraw(),
                Event::RedrawRequested(_) => redraw(
                    &layer,
                    &pipeline_state,
                    &command_queue,
                    &vertex_buffer,
                    &viewport_size_buffer,
                    &texture,
                ),
                _ => {}
            }
        })
    })
}
