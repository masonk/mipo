use anyhow::Result;

use bevy::render::{
    mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues},
    render_asset::RenderAssetUsages,
};

use crate::rtin::*;
use log::info;
use std::path::Path;

#[derive(Debug, Default, Clone, Copy)]
pub struct MeshOptions {
    pub error_threshold: f32,
}

pub fn load_mesh<P: AsRef<Path>>(path: P, options: MeshOptions) -> Result<(Mesh, MeshData)> {
    let rtin = preprocess_heightmap_from_img_path(path)?;
    let mesh_data = thresholded_mesh_data(options.error_threshold, &rtin);
    info!("Extracted a mesh with {} vertices", mesh_data.indices.len());
    let mesh = make_mesh(&mesh_data, &options);
    Ok((mesh, mesh_data))
}

pub fn make_mesh(mesh_data: &MeshData, _options: &MeshOptions) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let indices_len = mesh_data.indices.len();

    vertices.reserve(mesh_data.vertices.len());
    colors.reserve(vertices.len());
    indices.reserve(indices_len);

    let g = colorgrad::viridis();

    for vertex in &mesh_data.vertices {
        vertices.push([vertex.x, vertex.z, vertex.y]);

        let (r, g, b, a) = g.at(vertex[2] as f64).to_linear_rgba();
        colors.push([r as f32, g as f32, b as f32, a as f32]);
        // let color = g.at(vertex[2] as f64).to_array().map(|f| f as f32);
        // colors.push(color);
    }
    let triangle_number = mesh_data.indices.len() / 3;

    for i in 0..triangle_number {
        for j in 0..3 {
            indices.push(mesh_data.indices[i * 3 + j]);
        }
    }
    info!("Computing positions for {} vertices", vertices.len());
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(vertices),
    );

    info!("Computed {} color values", colors.len());
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::Float32x4(colors),
    );

    info!("Computed {} indices", indices.len());
    mesh.insert_indices(Indices::U32(indices));

    mesh.compute_smooth_normals();

    mesh
}
