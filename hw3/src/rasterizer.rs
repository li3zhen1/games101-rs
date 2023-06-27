use crate::texture::Texture;
use crate::triangle;
use crate::utils::image::{save_image, save_image_from_u8array};
use crate::utils::render::TextureConvertible;
use crate::{shader::*, triangle::Triangle, utils::render::KeyboardHandler};
use bitflags::bitflags;
use glam::*;
use std::{collections::HashMap, ffi::c_void};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BufferKind: u32 {
        const Color = 0b00000001;
        const Depth = 0b00000010;
        const All = Self::Color.bits() | Self::Depth.bits();
    }
}

#[inline]
fn compute_barycentric_2d(x: f32, y: f32, v: &[Vec4; 3]) -> Vec3 {
    let c1 = (x * (v[1].y - v[2].y) + (v[2].x - v[1].x) * y + v[1].x * v[2].y - v[2].x * v[1].y)
        / (v[0].x * (v[1].y - v[2].y) + (v[2].x - v[1].x) * v[0].y + v[1].x * v[2].y
            - v[2].x * v[1].y);
    let c2 = (x * (v[2].y - v[0].y) + (v[0].x - v[2].x) * y + v[2].x * v[0].y - v[0].x * v[2].y)
        / (v[1].x * (v[2].y - v[0].y) + (v[0].x - v[2].x) * v[1].y + v[2].x * v[0].y
            - v[0].x * v[2].y);
    let c3 = (x * (v[0].y - v[1].y) + (v[1].x - v[0].x) * y + v[0].x * v[1].y - v[1].x * v[0].y)
        / (v[2].x * (v[0].y - v[1].y) + (v[1].x - v[0].x) * v[2].y + v[0].x * v[1].y
            - v[1].x * v[0].y);
    vec3(c1, c2, c3)
}

pub enum PrimitiveKind {
    Line,
    Triangle,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct PosBufId(usize);

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct IndBufId(usize);

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct ColBufId(usize);

pub struct Rasterizer {
    w: usize,
    h: usize,

    next_id: usize,
    pos_buf: HashMap<PosBufId, Vec<Vec3>>,
    ind_buf: HashMap<IndBufId, Vec<UVec3>>,
    col_buf: HashMap<ColBufId, Vec<Vec4>>,
    nor_buf: HashMap<ColBufId, Vec<Vec4>>,

    tex_coord: HashMap<PosBufId, Vec<Vec2>>,

    frame_buf: Vec<Vec4>,
    frame_buf_supersampled: Vec<Vec4>,
    depth_buf_supersampled: Vec<f32>,

    model: Mat4,
    view: Mat4,
    projection: Mat4,

    antialiasing: usize,

    texture: Option<Texture>,
    vertex_shader: VertexShaderFn,
    fragment_shader: FragmentShaderFn,
}

impl Rasterizer {
    fn get_next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn new(w: usize, h: usize, antialiasing: usize) -> Self {
        Self {
            w,
            h,
            next_id: 0,
            pos_buf: HashMap::new(),
            ind_buf: HashMap::new(),
            col_buf: HashMap::new(),
            nor_buf: HashMap::new(),

            tex_coord: HashMap::new(),

            frame_buf: vec![vec4(0., 0., 0., 1.); w * h],
            frame_buf_supersampled: vec![vec4(0., 0., 0., 1.); w * h * antialiasing * antialiasing],
            // with antialiasing size?
            depth_buf_supersampled: vec![f32::INFINITY; w * h * antialiasing * antialiasing],

            model: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
            projection: Mat4::IDENTITY,
            antialiasing,

            texture: None,

            vertex_shader: |_| vec3(0., 0., 0.),
            fragment_shader: |_| vec4(0., 0., 0., 1.),
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

    pub fn load_colors(&mut self, col_buf: Vec<Vec4>) -> ColBufId {
        let id = ColBufId(self.get_next_id());
        self.col_buf.insert(id, col_buf);
        id
    }

    pub fn load_normals(&mut self, normals: Vec<Vec4>) -> ColBufId {
        let id = ColBufId(self.get_next_id());
        self.nor_buf.insert(id, normals);
        id
    }

    pub fn load_tex_coords(&mut self, tex_coords: Vec<Vec2>) -> PosBufId {
        let id = PosBufId(self.get_next_id());
        self.tex_coord.insert(id, tex_coords);
        id
    }

    pub fn clear(&mut self, kind: BufferKind) {
        if kind.contains(BufferKind::Color) {
            self.frame_buf_supersampled.fill(vec4(0., 0., 0., 1.));
        }
        if kind.contains(BufferKind::Depth) {
            self.depth_buf_supersampled.fill(f32::INFINITY);
        }
    }

    pub fn draw(
        &mut self,
        pos_buf: &PosBufId,
        ind_buf: &IndBufId,
        col_buf: &ColBufId,
        primitive_kind: PrimitiveKind,
    ) {
        match primitive_kind {
            PrimitiveKind::Triangle => {
                let triangles: Vec<Triangle> = {
                    let buf = &self.pos_buf[pos_buf];
                    let ind = &self.ind_buf[ind_buf];
                    let col = &self.col_buf[col_buf];

                    let f1 = 99.9f32 / 2.0;
                    let f2 = 100.1f32 / 2.0;

                    let mvp = self.projection * self.view * self.model;

                    ind.into_iter()
                        .map(|vi: &UVec3| {
                            let mut t = Triangle::zeros();

                            vec![vi.x, vi.y, vi.z]
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
                                .for_each(|it| {
                                    t.set_vertex(it.0, it.1);
                                });

                            vec![vi.x, vi.y, vi.z]
                                .iter()
                                .enumerate()
                                .for_each(|(n, &it)| {
                                    let color = col[it as usize];
                                    t.set_color_rgb(n, color.x, color.y, color.z);
                                });
                            t
                        })
                        .collect()
                };

                for (i, t) in triangles.iter().enumerate() {
                    self.rasterize_triangle_antialiased(&t);
                }

                // let u8array: Vec<u8> = self
                //     .depth_buf_supersampled
                //     .iter()
                //     .flat_map(|it| {
                //         [
                //             (it + 2.) as u8,
                //             (it + 2.) as u8,
                //             (it + 2.) as u8,
                //             (it + 2.) as u8,
                //         ]
                //     })
                //     .collect();

                // save_image_from_u8array(
                //     &u8array,
                //     (self.w * self.antialiasing) as u32,
                //     (self.h * self.antialiasing) as _,
                //     "image2.png",
                // );
                // exit(0);

                let sampling_count = (self.antialiasing * self.antialiasing) as f32;

                for i in 0..self.w {
                    for j in 0..self.h {
                        let mut color = vec4(0., 0., 0., 1.);
                        for k in 0..self.antialiasing {
                            for l in 0..self.antialiasing {
                                let idx = (i * self.antialiasing + k)
                                    + (j * self.antialiasing + l) * self.w * self.antialiasing;
                                color += self.frame_buf_supersampled[idx];
                            }
                        }
                        color /= sampling_count;
                        self.frame_buf[i + j * self.w] = color;
                    }
                }
            }
            PrimitiveKind::Line => {}
        }
    }

    pub fn draw_triangle_list(&mut self, triangles: &Vec<Triangle>) {
        let f1 = 99.9f32 / 2.;
        let f2 = 100.1f32 / 2.;
        let vm = self.view * self.model;
        let mvp = self.projection * self.view * self.model;

        let triangles: Vec<(Triangle, _)> = triangles
            .iter()
            .map(|t| {

                let viewspace_pos = t.v.map(|it| (vm * it).xyz());

                let mut t: Triangle = t.clone();

                let transformed_vertex =
                    t.v.iter()
                        .map(|&v| {
                            let mut vec = mvp * v;
                            vec /= vec.w;
                            vec.x = 0.5 * (self.w as f32) * (vec.x + 1.0);
                            vec.y = 0.5 * (self.h as f32) * (vec.y + 1.0);
                            vec.z = vec.z * f1 + f2;
                            vec
                        })
                        .enumerate()
                        .collect::<Vec<_>>();

                transformed_vertex.iter().for_each(|it| {
                    t.set_vertex(it.0, it.1);
                });
                
                (t, viewspace_pos)
            })
            .collect::<_>();

        for (t, viewspace_pos) in triangles {
            self.rasterize_triangle_antialiased_with_shader(&t, &viewspace_pos);
        }

        let sampling_count = (self.antialiasing * self.antialiasing) as f32;

        for i in 0..self.w {
            for j in 0..self.h {
                let mut color = vec4(0., 0., 0., 1.);
                for k in 0..self.antialiasing {
                    for l in 0..self.antialiasing {
                        let idx = (i * self.antialiasing + k)
                            + (j * self.antialiasing + l) * self.w * self.antialiasing;
                        color += self.frame_buf_supersampled[idx];
                    }
                }
                color /= sampling_count;
                self.frame_buf[i + j * self.w] = color;
            }
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

    // pub fn rasterize_wireframe(&mut self, t: &Triangle) {
    //     self.draw_line(t.a(), t.b());
    //     self.draw_line(t.b(), t.c());
    //     self.draw_line(t.c(), t.a());
    // }

    #[inline]
    pub fn set_pixel(&mut self, point: (usize, usize), color: Vec4) {
        if point.0 >= self.w || point.1 >= self.h {
            return;
        };

        let i = (self.h - point.1) * self.w + point.0;
        self.frame_buf[i] = color;
    }

    #[inline]
    pub fn set_depth(&mut self, point: (usize, usize), depth: f32) {
        if point.0 >= self.w || point.1 >= self.h {
            return;
        };

        let i = (self.h - point.1) * self.w + point.0;
        self.depth_buf_supersampled[i] = depth;
    }

    // #[inline]
    // pub fn set_pixel_nocheck(&mut self, i: usize, color: Vec4) {
    //     self.frame_buf[i] = color;
    // }

    // #[inline]
    // pub fn set_depth_nocheck(&mut self, i: usize, depth: f32) {
    //     self.depth_buf[i] = depth;
    // }

    pub fn get_index(&self, x: usize, y: usize) -> usize {
        return (self.h - y) * self.w + x;
    }

    pub fn get_index_for_antialiased(&self, x: usize, y: usize) -> usize {
        return (self.h * self.antialiasing - y) * self.w * self.antialiasing + x;
    }

    fn get_sample_points(&self, x: f32, y: f32) -> Vec<(f32, f32, usize, usize)> {
        let half_subpixel_size = 0.5 / self.antialiasing as f32;
        let mut points = vec![];
        for i in 0..self.antialiasing {
            for j in 0..self.antialiasing {
                let x = x + (i as f32 * 2. + 1.) * half_subpixel_size;
                let y = y + (j as f32 * 2. + 1.) * half_subpixel_size;
                points.push((
                    x,
                    y,
                    i + self.antialiasing * x as usize,
                    j + self.antialiasing * y as usize,
                ));
            }
        }
        points
    }

    fn rasterize_triangle_antialiased(&mut self, t: &Triangle) {
        let bbox = t.bounding_box();
        // let antialiasing = self.antialiasing as f32;

        let points: Vec<(f32, f32, usize, usize)> = bbox
            .x_range()
            .flat_map(|x| bbox.y_range().map(move |y| (x as f32, y as f32)))
            .flat_map(|(i, j)| self.get_sample_points(i as _, j as _))
            .collect();

        for (x, y, i, j) in points {
            let c = compute_barycentric_2d(x, y, &t.v);
            let inside = c.x >= 0. && c.y >= 0. && c.z >= 0.;

            if !inside {
                continue;
            }

            if i >= self.w * self.antialiasing || j >= self.h * self.antialiasing {
                continue;
            }

            let supersampled_index = self.get_index_for_antialiased(i, j);
            let depth = t.v[0].z * c.x + t.v[1].z * c.y + t.v[2].z * c.z;
            if depth < self.depth_buf_supersampled[supersampled_index] {
                let colors = Mat3::from_cols(t.color[0].xyz(), t.color[1].xyz(), t.color[2].xyz());
                let color = (colors * c).extend(1.);
                self.frame_buf_supersampled[supersampled_index] = color;
                self.depth_buf_supersampled[supersampled_index] = depth;
            }
        }
    }

    fn rasterize_triangle_antialiased_with_shader(&mut self, t: &Triangle, view_pos: &[Vec3; 3]) {
        let bbox = t.bounding_box();
        // let antialiasing = self.antialiasing as f32;

        let points: Vec<(f32, f32, usize, usize)> = bbox
            .x_range()
            .flat_map(|x| bbox.y_range().map(move |y| (x as f32, y as f32)))
            .flat_map(|(i, j)| self.get_sample_points(i as _, j as _))
            .collect();

        match &self.texture {
            Some(texture) => {
                for (x, y, i, j) in points {
                    let c = compute_barycentric_2d(x, y, &t.v);
                    let inside = c.x >= 0. && c.y >= 0. && c.z >= 0.;

                    if !inside {
                        continue;
                    }

                    if i >= self.w * self.antialiasing || j >= self.h * self.antialiasing {
                        continue;
                    }

                    let supersampled_index = self.get_index_for_antialiased(i, j);
                    let depth = t.v[0].z * c.x + t.v[1].z * c.y + t.v[2].z * c.z;
                    if depth < self.depth_buf_supersampled[supersampled_index] {
                        let texcoord = vec2(
                            t.tex_coords[0].x * c[0]
                                + t.tex_coords[1].x * c[1]
                                + t.tex_coords[2].x * c[2],
                            t.tex_coords[0].y * c[0]
                                + t.tex_coords[1].y * c[1]
                                + t.tex_coords[2].y * c[2],
                        );

                        let color = texture.get_color_by_tex_coord(texcoord);
                        self.frame_buf_supersampled[supersampled_index] = color.extend(1.);

                        self.depth_buf_supersampled[supersampled_index] = depth;
                    }
                }
            }
            None => {
                panic!("Referencing empty texture");
            }
        }
    }

    pub fn set_texture(&mut self, texture: Texture) {
        self.texture = Some(texture);
    }

    pub fn set_vertex_shader(&mut self, vertex_shader: VertexShaderFn) {
        self.vertex_shader = vertex_shader;
    }

    pub fn set_fragment_shader(&mut self, fragment_shader: FragmentShaderFn) {
        self.fragment_shader = fragment_shader;
    }

    // pub fn rasterize_triangle(&mut self, t: &Triangle) {
    //     let bbox = t.bounding_box();
    //     for i in bbox.x_range() {
    //         for j in bbox.y_range() {
    //             if i >= self.w || j >= self.h {
    //                 continue;
    //             };

    //             let c = compute_barycentric_2d(i as f32 + 0.5, j as f32 + 0.5, &t.v);

    //             let inside = c.x >= 0. && c.y >= 0. && c.z >= 0.;

    //             if inside {
    //                 let depth = t.v[0].z * c.x + t.v[1].z * c.y + t.v[2].z * c.z;
    //                 let index = self.get_index(i, j);

    //                 if depth < self.depth_buf[index] {
    //                     let colors =
    //                         Mat3::from_cols(t.color[0].xyz(), t.color[1].xyz(), t.color[2].xyz());
    //                     let color = (colors * c).extend(1.);

    //                     self.set_depth_nocheck(index, depth);
    //                     self.set_pixel_nocheck(index, color);
    //                 }
    //             }
    //         }
    //     }
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
                    (it.w * 255.0) as u8,
                ]
            })
            .collect()
    }
}
