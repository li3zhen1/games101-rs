use cocoa::{appkit::NSView, base::id as cocoa_id};
use core_graphics_types::geometry::CGSize;
use metal::{
    Buffer, CommandQueue, Device, Library, MTLClearColor, MTLLoadAction, MTLOrigin, MTLPixelFormat,
    MTLPrimitiveType, MTLRegion, MTLResourceOptions, MTLSize, MTLStoreAction, MTLTextureUsage,
    MetalLayer, MetalLayerRef, RenderPassDescriptor, RenderPassDescriptorRef,
    RenderPipelineDescriptor, RenderPipelineState, Texture, TextureDescriptor, TextureRef,
};
use std::ffi::c_void;
use std::path::PathBuf;

use winit::dpi::LogicalSize;
use winit::event::{WindowEvent, KeyboardInput, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::macos::WindowExtMacOS;
use winit::window::Window;

use super::shader_types::*;


pub fn create_window(event_loop: &EventLoop<()>, width: u32, height: u32, title: &str) -> Window {
    let window_size = LogicalSize::new(width, height);

    winit::window::WindowBuilder::new()
        .with_inner_size(window_size)
        .with_title(title)
        .build(event_loop)
        .unwrap()
}

pub fn get_window_layer(
    window: &Window,
    device: &Device,
    pixel_format: MTLPixelFormat,
) -> MetalLayer {
    let layer = MetalLayer::new();

    layer.set_device(device);
    layer.set_pixel_format(pixel_format);
    layer.set_presents_with_transaction(false);
    let inner_size = window.inner_size();
    layer.set_drawable_size(CGSize {
        width: inner_size.width as f64,
        height: inner_size.height as f64,
    });

    unsafe {
        let view_id = window.ns_view() as cocoa_id;
        view_id.setWantsLayer(true);
        view_id.setLayer(std::mem::transmute(layer.as_ref()));
    };

    layer
}

pub fn get_metal_lib(metal_lib_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(metal_lib_name)
}

pub fn get_vertices(width: f32, height: f32) -> [TexturedVertex; 6] {
    // println!("Vertex: {width}, {height}");
    let w = width as f32 / 2.;
    let h = height as f32 / 2.;
    [
        TexturedVertex::new(-w, -h, 0., 1.),
        TexturedVertex::new(w, -h, 1., 1.),
        TexturedVertex::new(w, h, 1., 0.),
        TexturedVertex::new(-w, -h, 0., 1.),
        TexturedVertex::new(w, h, 1., 0.),
        TexturedVertex::new(-w, h, 0., 0.),
    ]
}

pub fn update_viewport_size(viewport_size_buffer: &Buffer, viewport_size: [u32; 2]) {
    // println!("{}, {}", viewport_size[0], viewport_size[1]);
    let contents = viewport_size_buffer.contents();

    let byte_count = (viewport_size.len() * std::mem::size_of::<u32>()) as usize;

    unsafe {
        std::ptr::copy(viewport_size.as_ptr(), contents as *mut u32, byte_count);
    };

    // viewport_size_buffer.did_modify_range(
    //     NSRange::new(0, byte_count as u64)
    // );
}

pub trait TextureConvertible {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn contents(&self) -> *const c_void;
    fn bytes_per_pixel(&self) -> usize;
    fn dump_u8norm(&self) -> Vec<u8>;
}

pub fn create_texture<T: TextureConvertible>(
    source: &T,
    device: &Device,
    pixel_format: MTLPixelFormat,
) -> Texture {
    
    let w = source.width() as u64;
    let h = source.height() as u64;
    let bytes = source.contents();

    let texture = TextureDescriptor::new();
    texture.set_width(w);
    texture.set_height(h);
    texture.set_pixel_format(pixel_format);
    texture.set_usage(MTLTextureUsage::ShaderRead);

    let texture = device.new_texture(&texture);

    texture.replace_region(
        MTLRegion {
            origin: MTLOrigin { x: 0, y: 0, z: 0 },
            size: MTLSize {
                width: w,
                height: h,
                depth: 1,
            },
        },
        0,
        bytes,
        w * (source.bytes_per_pixel() as u64),
    );

    texture
}

pub fn update_texture<T: TextureConvertible>(
    src: &T, 
    dst: &Texture,
) {
    let w = src.width() as u64;
    dst.replace_region(
        MTLRegion {
            origin: MTLOrigin { x: 0, y: 0, z: 0 },
            size: MTLSize {
                width: w,
                height: src.height() as _,
                depth: 1,
            },
        },
        0,
        src.contents(),
        w * (src.bytes_per_pixel() as u64),
    );
}

pub fn prepare_pipeline_state(
    device: &Device,
    library: &Library,
    vertex_fn_name: &str,
    fragment_fn_name: &str,
    pixel_format: MTLPixelFormat,
) -> RenderPipelineState {
    let ps = RenderPipelineDescriptor::new();
    let vs = library.get_function(vertex_fn_name, None).unwrap();
    let fs = library.get_function(fragment_fn_name, None).unwrap();

    ps.set_vertex_function(Some(&vs));
    ps.set_fragment_function(Some(&fs));

    ps.color_attachments()
        .object_at(0)
        .unwrap()
        .set_pixel_format(pixel_format);

    device.new_render_pipeline_state(&ps).unwrap()
}

pub trait KeyboardHandler {
    fn handle_keyboard_input(&self, input: KeyboardInput);
}

pub fn handle_window_event<T: KeyboardHandler>(
    event: WindowEvent,
    control_flow: &mut ControlFlow,
    layer: &MetalLayerRef,
    vp_size_buffer: &Buffer,
    keyboard_handler: &T,
) {
    match event {
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        WindowEvent::Resized(size) => {
            layer.set_drawable_size(CGSize {
                width: size.width as f64,
                height: size.height as f64,
            });
            update_viewport_size(vp_size_buffer, [size.width, size.height]);
        }
        WindowEvent::KeyboardInput { input, .. } => {
            keyboard_handler.handle_keyboard_input(input);
        }
        _ => {}
    }
}

fn prepare_render_pass_descriptor(descriptor: &RenderPassDescriptorRef, texture: &TextureRef) {
    let color = descriptor.color_attachments().object_at(0).unwrap();

    color.set_texture(Some(texture));
    color.set_load_action(MTLLoadAction::Clear);
    color.set_clear_color(MTLClearColor::new(0.2, 0.5, 0.8, 1.0));
    color.set_store_action(MTLStoreAction::Store);
}

pub fn redraw(
    layer: &MetalLayerRef,
    pipeline_state: &RenderPipelineState,
    command_queue: &CommandQueue,
    vertex_buffer: &Buffer,
    viewport_size_buffer: &Buffer,
    texture: &TextureRef,
) {
    let drawable = match layer.next_drawable() {
        Some(drawable) => drawable,
        None => return,
    };

    let render_pass = RenderPassDescriptor::new();

    // use drawable_texture
    prepare_render_pass_descriptor(&render_pass, &drawable.texture());
    let command_buffer = command_queue.new_command_buffer();
    let encoder = command_buffer.new_render_command_encoder(&render_pass);

    encoder.set_render_pipeline_state(pipeline_state);
    encoder.set_vertex_buffer(VertexInput::Vertices as _, Some(vertex_buffer), 0);
    encoder.set_vertex_buffer(
        VertexInput::ViewportSize as _,
        Some(viewport_size_buffer),
        0,
    );
    encoder.set_fragment_texture(TextureIndex::BaseColor as _, Some(texture));

    encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, 6);
    encoder.end_encoding();

    command_buffer.present_drawable(&drawable);
    command_buffer.commit();
}

