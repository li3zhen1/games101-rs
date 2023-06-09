use crate::{triangle::Triangle, utils::render::KeyboardHandler};
use crate::utils::render::TextureConvertible;
use bitflags::bitflags;
use glam::*;
use std::{collections::HashMap, ffi::c_void, ops::Index};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BufferKind: u32 {
        const Color = 0b00000001;
        const Depth = 0b00000010;
        const All = Self::Color.bits() | Self::Depth.bits();
    }
}

pub enum PrimitiveKind {
    Line,
    Triangle,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct PosBufId(usize);

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct IndBufId(usize);

#[derive(Default)]
pub struct Rasterizer{
    w: usize,
    h: usize,

    next_id: usize,
    pos_buf: HashMap<PosBufId, Vec<Vec3>>,
    ind_buf: HashMap<IndBufId, Vec<UVec3>>,
    frame_buf: Vec<Vec4>, // Vec4<f16>, use transmute
    depth_buf: Vec<f32>,

    model: Mat4,
    view: Mat4,
    projection: Mat4,
}

impl Rasterizer {
    fn get_next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            next_id: 0,
            pos_buf: HashMap::new(),
            ind_buf: HashMap::new(),
            frame_buf: vec![vec4(0., 0., 0., 1.); w * h],
            depth_buf: vec![0.0; w * h],
            model: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
            projection: Mat4::IDENTITY,
        }
    }

    pub fn set_model(&mut self, model: Mat4) {
        self.model = model;
    }

    pub fn set_view(&mut self, view: Mat4) {
        self.view = view;
    }

    pub fn set_projection(&mut self, projection: Mat4) {
        self.projection = projection;
    }

    pub fn load_positions(&mut self, pos_buf: Vec<Vec3>) -> PosBufId {
        let id = PosBufId(self.get_next_id());
        self.pos_buf.insert(id, pos_buf);
        id
    }

    pub fn load_indices(&mut self, ind_buf: Vec<UVec3>) -> IndBufId {
        let id = IndBufId(self.get_next_id());
        self.ind_buf.insert(id, ind_buf);
        id
    }

    pub fn clear(&mut self, kind: BufferKind) {
        if kind.contains(BufferKind::Color) {
            self.frame_buf = vec![vec4(0., 0., 0., 1.); self.w * self.h];
        }
        if kind.contains(BufferKind::Depth) {
            self.depth_buf = vec![0.0; self.w * self.h];
        }
    }

    pub fn draw(&mut self, pos_buf: &PosBufId, ind_buf: &IndBufId, primitive_kind: PrimitiveKind) {
        match primitive_kind {
            PrimitiveKind::Triangle => {
                let triangles: Vec<Triangle> = {
                    let buf = &self.pos_buf[pos_buf];
                    let ind = &self.ind_buf[ind_buf];

                    let f1 = 99.9f32 / 2.0;
                    let f2 = 100.1f32 / 2.0;

                    let mvp = self.projection * self.view * self.model;

                    ind.into_iter()
                        .map(|i: &UVec3| {
                            let mut t = Triangle::zeros();

                            vec![i.x, i.y, i.z]
                                .iter()
                                .map(|&i| {
                                    let mut vec = mvp * (buf[i as usize].extend(1.0));
                                    vec /= vec.w;
                                    vec.x = 0.5 * (self.w as f32) * (vec.x + 1.0);
                                    vec.y = 0.5 * (self.h as f32) * (vec.y + 1.0);
                                    vec.z = vec.z * f1 + f2;
                                    vec
                                })
                                .enumerate()
                                .for_each(|it| t.set_vertex(it.0, it.1.xyz()));

                            t.set_color_rgb(0, 1.0, 0.0, 0.0);
                            t.set_color_rgb(1, 0.0, 1.0, 0.0);
                            t.set_color_rgb(2, 0.0, 0.0, 1.0);
                            t
                        })
                        .collect()
                };

                for t in triangles {
                    self.rasterize_wireframe(&t);
                }
            }
            PrimitiveKind::Line => {}
        }
    }

    pub fn draw_line(&mut self, begin: Vec3, end: Vec3) {
        let color = vec4(1., 1., 1., 1.);

        let x0 = begin.x;
        let y0 = begin.y;
        let x1 = end.x;
        let y1 = end.y;

        // TODO: inf?
        let k = (y1 - y0) / (x1 - x0);

        let steep = (x0 - x1).abs() < (y0 - y1).abs();

        let x0 = begin.x as usize;
        let y0 = begin.y as usize;
        let x1 = end.x as usize;
        let y1 = end.y as usize;

        if steep {
            // iter by y

            let range = if y1 > y0 { y0..y1 + 1 } else { y1..y0 + 1 };

            for y in range {
                let x = ((y as f32 - y0 as f32) / k + x0 as f32) as usize;
                self.set_pixel((x, y), color);
            }
        } else {
            // iter by x
            let range = if x1 > x0 { x0..x1 + 1 } else { x1..x0 + 1 };

            for x in range {
                let y = (k * (x as f32 - x0 as f32) + (y0 as f32)) as usize;
                self.set_pixel((x, y), color);
            }
        }
    }

    pub fn rasterize_wireframe(&mut self, t: &Triangle) {
        self.draw_line(t.a(), t.b());
        self.draw_line(t.b(), t.c());
        self.draw_line(t.c(), t.a());
    }

    pub fn set_pixel(&mut self, point: (usize, usize), color: Vec4) {
        if point.0 >= self.w || point.1 >= self.h {
            return;
        };

        let i = (self.h - point.1) * self.w + point.0;
        self.frame_buf[i] = color;
    }

    pub fn get_index(&self, x: usize, y: usize) -> usize {
        return (self.h - y) * self.w + x;
    }

    // pub fn dump_pixels_RGBA8Unorm(&self) -> Vec<u8> {
    //     self.frame_buf
    //         .iter()
    //         .flat_map(|it| {
    //             [
    //                 (it.x * 255.0) as u8,
    //                 (it.y * 255.0) as u8,
    //                 (it.z * 255.0) as u8,
    //                 255,
    //             ]
    //         })
    //         .collect()
    // }
}

impl TextureConvertible for Rasterizer {
    fn width(&self) -> usize {
        self.w
    }

    fn height(&self) -> usize {
        self.h
    }

    fn contents(&self) -> *const c_void {
        self.frame_buf.as_ptr() as _
    }

    fn bytes_per_pixel(&self) -> usize {
        std::mem::size_of::<Vec4>()
    }

    fn dump_u8norm(&self) -> Vec<u8> {
        self.frame_buf
            .iter()
            .flat_map(|it| {
                [
                    (it.x * 255.0) as u8,
                    (it.y * 255.0) as u8,
                    (it.z * 255.0) as u8,
                    255,
                ]
            })
            .collect()
    }
}

impl KeyboardHandler for Rasterizer {
    fn handle_keyboard_input(&self, input: winit::event::KeyboardInput) {
        
    }
}
