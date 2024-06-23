use bevy::{pbr::wireframe::Wireframe, prelude::*};

use crate::{bevy_rtin, bevy_rtin::MeshOptions};
use bevy_rapier3d::math::Vect;
use bevy_rapier3d::prelude::*;
use std::path::PathBuf;

pub struct WorldPlugin {
    pub(crate) terrain_path: PathBuf,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                make_spawn_floor(self.terrain_path.clone()),
                spawn_light,
                spawn_objects,
            ),
        );
    }
}

fn make_spawn_floor(
    terrain_path: PathBuf,
) -> impl FnMut(Commands, ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>) {
    move |mut commands, mut meshes, mut materials| {
        let (shaded, shaded_mesh_data) =
            bevy_rtin::load_mesh(&terrain_path, MeshOptions::default()).unwrap();

        let parry3d_vertices: Vec<Vect> = shaded_mesh_data
            .vertices
            .into_iter()
            .map(|v| Vect::new(v[0], v[2], v[1]))
            .collect();

        let parry3d_indices = shaded_mesh_data
            .indices
            .into_iter()
            .array_chunks::<3>()
            .collect();

        let collider = Collider::trimesh(parry3d_vertices, parry3d_indices);

        let shaded_handle = meshes.add(shaded);
        let mat = StandardMaterial {
            cull_mode: None,
            unlit: false,
            metallic: 0.,
            perceptual_roughness: 0.5,
            base_color: Color::WHITE,
            ..default()
        };
        let white_material = materials.add(mat);

        commands
            .spawn((
                PbrBundle {
                    mesh: shaded_handle,
                    material: white_material.clone(),
                    transform: Transform::from_scale(Vec3::new(1., 50.0, 1.0)),
                    ..default()
                },
                Wireframe,
                RigidBody::Fixed,
                Name::new("shaded_floor"),
            ))
            .with_children(|p| {
                p.spawn((
                    Name::new("terrain_collider"),
                    collider,
                    Wireframe,
                    TransformBundle {
                        local: Transform::from_scale(Vec3::new(1., 1.0, 1.0)),
                        ..default()
                    },
                ));
            });
    }
}
fn spawn_light(mut commands: Commands) {
    let grid_size = 250.;
    let light_grid_size = 10;
    let interval = grid_size / light_grid_size as f32;

    for x in 0..light_grid_size {
        for z in 0..light_grid_size {
            let transform = Transform::from_xyz(x as f32 * interval, 150., z as f32 * interval);
            commands.spawn((
                PointLightBundle {
                    point_light: PointLight {
                        color: Color::rgba(1.0, 1.0, 1.0, 1.000),
                        intensity: 2e8,
                        range: 500.,
                        radius: 25.,
                        shadows_enabled: true,
                        ..default()
                    },
                    transform,
                    ..default()
                },
                Name::new(format!("world_light_{x}_{z}")),
            ));
        }
    }
}

fn spawn_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut cube = |size: f32,
                    material: Handle<StandardMaterial>,
                    transform: Transform,
                    name: String|
     -> (PbrBundle, Name) {
        (
            PbrBundle {
                mesh: meshes.add(Mesh::from(Cuboid::new(size, size, size))),
                material: material,
                transform: transform,
                ..default()
            },
            Name::new(name),
        )
    };

    let blue_cube = cube(
        4.0,
        materials.add(Color::BLUE),
        Transform::from_xyz(-5.5, 3.1, 5.5),
        "cube_blue".to_string(),
    );

    let red_cube = cube(
        2.0,
        materials.add(Color::RED),
        Transform::from_xyz(5.0, 0.7, -1.1),
        "cube_red".to_string(),
    );

    commands.spawn(blue_cube);
    commands.spawn(red_cube);
}
