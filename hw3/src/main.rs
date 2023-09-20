mod rasterizer;
mod loader;
mod transform;
mod triangle;
mod utils;
mod texture;
mod shader;
use std::env;

use crate::rasterizer::Rasterizer;
use crate::transform::*;

use core_graphics::geometry::CGSize;
use glam::{uvec3, vec3, vec4};
use loader::load_obj;
use metal::{Device, MTLPixelFormat, MTLResourceOptions};
use rasterizer::{BufferKind, PrimitiveKind};
use shader::{vertex_shader, phong_fragment_shader};
use texture::Texture;
use utils::{image::save_image, render::*, shader_types::TexturedVertex};
use winit::{
    event::{VirtualKeyCode, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
};


use objc::rc::autoreleasepool;
use winit::event::Event;

const TEXTURE_PIXEL_FORMAT: MTLPixelFormat = MTLPixelFormat::RGBA32Float;

// CAMetalLayer only accepts the following pixel formats:
// https://developer.apple.com/documentation/quartzcore/cametallayer/1478155-pixelformat
// RGBA16Float displays wrong colors
const PIPE_LINE_PIXEL_FORMAT: MTLPixelFormat = MTLPixelFormat::RGBA8Unorm;

const VERTEX_SHADER_NAME: &str = "quad_vertex";
const FRAGMENT_SHADER_NAME: &str = "sampling_shader";
const SHADER_FILE_NAME: &str = env!("SHADER_FILE_NAME");

const INITIAL_WINDOW_WIDTH: u32 = 700;
const INITIAL_WINDOW_HEIGHT: u32 = 700;
const DELTA_ANGLE: f32 = 1.;



const ASSET_PATH: &str = "models/spot/";
const MODEL_PATH: &str = "models/spot/spot_triangulated_good.obj";
const TEXTURE_PATH: &str = "models/spot/spot_texture.png";

fn main() {
    let args: Vec<String> = env::args().collect();

    let dump_image = args.len() >= 3;

    let mut r = Rasterizer::new(INITIAL_WINDOW_WIDTH as _, INITIAL_WINDOW_HEIGHT as _, 2);
    
    
    let triangle_lists = loader::load_obj(MODEL_PATH);
    r.set_texture(Texture::new(TEXTURE_PATH));
    r.set_vertex_shader(vertex_shader);
    r.set_fragment_shader(phong_fragment_shader);
    let eye_pos = vec3(0.0, 0.0, 10.0);
    let mut angle = 0f32;

    r.clear(BufferKind::Color | BufferKind::Depth);

    r.set_model(get_model_matrix(angle));
    r.set_view(get_view_matrix(eye_pos));
    r.set_projection(get_projection_matrix(45.0, 1.0, 0.1, 50.0));

    r.draw_triangle_list(&triangle_lists[0]);
    
    if dump_image {
        let angle = match args.get(2) {
            Some(arg) => arg.parse::<f32>().unwrap_or(0.),
            _ => 0.,
        };
        r.set_model(get_model_matrix(angle));
        save_image(&r, args.get(3).unwrap_or(&String::from("output.png")));
        return; 
    }

    let event_loop = EventLoop::new();
    let window = create_window(
        &event_loop,
        INITIAL_WINDOW_WIDTH,
        INITIAL_WINDOW_HEIGHT,
        "HW3",
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
        vertex_data.as_ptr() as _,
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


    let mut angle_changed = true;

    event_loop.run(move |event, _, control_flow| {
        autoreleasepool(|| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(size) => {
                        layer.set_drawable_size(CGSize {
                            width: size.width as f64,
                            height: size.height as f64,
                        });
                        update_viewport_size(&viewport_size_buffer, [size.width, size.height]);
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.state == ElementState::Pressed {
                            return;
                        };
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::A) => {
                                angle += DELTA_ANGLE;
                                angle_changed = true;
                                // println!("angle: {}", angle);
                            }
                            Some(VirtualKeyCode::D) => {
                                angle -= DELTA_ANGLE;
                                angle_changed = true;
                                // println!("angle: {}", angle);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
                Event::MainEventsCleared => window.request_redraw(),
                Event::RedrawRequested(_) => {
                    if !angle_changed {
                        return;
                    }
                    r.clear(BufferKind::Color | BufferKind::Depth);
                    r.set_model(get_model_matrix(angle));
                    r.draw_triangle_list(&triangle_lists[0]);
                    update_texture(&r, &texture);
                    redraw(
                        &layer,
                        &pipeline_state,
                        &command_queue,
                        &vertex_buffer,
                        &viewport_size_buffer,
                        &texture,
                    );
                    angle_changed = false;
                }

                _ => {}
            }
        })
    })
}
