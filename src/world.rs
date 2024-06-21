use bevy::prelude::*;

use crate::{bevy_rtin, bevy_rtin::MeshOptions};
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_floor, spawn_light, spawn_objects));
    }
}

fn spawn_floor(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let shaded = bevy_rtin::load_mesh(
        "assets/grand_canyon_small_heightmap.png",
        MeshOptions::default(),
    )
    .unwrap();

    let wireframe = bevy_rtin::load_mesh(
        "assets/grand_canyon_small_heightmap.png",
        MeshOptions {
            wireframe: true,
            error_threshold: 0.1,
        },
    )
    .unwrap();
    let shaded_handle = meshes.add(shaded);
    let wireframe_handle = meshes.add(wireframe);

    // commands.spawn((
    //     PbrBundle {
    //         mesh: shaded_handle,
    //         transform: Transform::from_scale(Vec3::new(1., 150.0, 1.0)),
    //         ..default()
    //     },
    //     Name::new("shaded_floor"),
    // ));
    commands.spawn((
        PbrBundle {
            mesh: wireframe_handle,
            transform: Transform::from_scale(Vec3::new(1., 50.0, 1.0)),
            ..default()
        },
        Name::new("wireframe_floor"),
    ));
}
fn spawn_light(mut commands: Commands) {
    let light = (
        PointLightBundle {
            point_light: PointLight {
                color: Color::rgba(0.835, 0.171, 0.171, 1.000),
                intensity: 100000.0,
                range: 29.0,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 5.0, 0.0),
            ..default()
        },
        Name::new("world_light"),
    );
    commands.spawn(light);
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
