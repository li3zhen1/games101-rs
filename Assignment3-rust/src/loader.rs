use crate::{rasterizer::Rasterizer, triangle::Triangle};
use glam::*;

pub fn load_obj(obj_path: &str) -> Vec<Vec<Triangle>> {
    let (models, materials) = tobj::load_obj(obj_path, &tobj::GPU_LOAD_OPTIONS).unwrap();

    models
        .iter()
        .map(|model| {
            let mesh = &model.mesh;
            mesh.indices
                .chunks(3)
                .map(|index_list| {
                    let index_list = [
                        index_list[0] as usize,
                        index_list[1] as _,
                        index_list[2] as _,
                    ];
                    Triangle {
                        v: index_list.map(|index| {
                            vec4(
                                mesh.positions[3 * index],
                                mesh.positions[3 * index + 1],
                                mesh.positions[3 * index + 2],
                                1.,
                            )
                        }),
                        color: [
                            vec4(0., 0., 0., 1.),
                            vec4(0., 0., 0., 1.),
                            vec4(0., 0., 0., 1.),
                        ],
                        tex_coords: index_list.map(|index| {
                            vec2(mesh.texcoords[2 * index], mesh.texcoords[2 * index + 1])
                        }),
                        normal: index_list.map(|index| {
                            vec3(
                                mesh.normals[3 * index],
                                mesh.normals[3 * index + 1],
                                mesh.normals[3 * index + 2],
                            )
                        }),
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<_>()
}

impl Rasterizer {
    pub fn load_obj(&mut self, obj_path: &str) {
        // let obj_path = "models/spot/spot_triangulated_good.obj";
        // > positions: (9675) vec![0.31728
        // > vertex_color: (0) vec! []
        // > normals: (9675) vec![0.57446701
        // > texcoords: (6450) vec! [0.80037.
        // > indices: (17568) vec![0, 1, 2,
        // > face arities: (0) vec![]
        // > texcoord_indices: (0) vec! []
        // > normal_indices: (0) vec![]

        let (models, materials) = tobj::load_obj(obj_path, &tobj::GPU_LOAD_OPTIONS).unwrap();

        for model in &models {
            let mesh = &model.mesh;

            let indices = mesh
                .indices
                .chunks(3)
                .map(|v| UVec3 {
                    x: v[0],
                    y: v[1],
                    z: v[2],
                })
                .collect::<Vec<_>>();
            let ind_id = self.load_indices(indices);

            let positions = mesh
                .positions
                .chunks(3)
                .map(|v| vec3(v[0], v[1], v[2]))
                .collect::<Vec<_>>();
            let pos_id = self.load_positions(positions);

            let normals = mesh
                .normals
                .chunks(3)
                .map(|v| vec4(v[0], v[1], v[2], 1.0))
                .collect::<Vec<_>>();

            let normal_id = self.load_normals(normals);

            let texcoords = mesh
                .texcoords
                .chunks(2)
                .map(|v| vec2(v[0], v[1]))
                .collect::<Vec<_>>();

            let texcoords_id = self.load_tex_coords(texcoords);
        }

        match materials {
            Ok(materials) => {
                println!("{:?} materials unhandled", materials.len());
            }
            _ => {}
        }
    }
}
