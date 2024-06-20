use bevy::{prelude::*, transform};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_floor, spawn_light, spawn_objects));
    }
}

fn spawn_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor = (
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(15., 15.)),
            material: materials.add(Color::DARK_GREEN),
            ..default()
        },
        Name::new("floor"),
    );

    commands.spawn(floor);
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
