use bevy::{pbr::wireframe::WireframeConfig, prelude::*};

use crate::{bevy_rtin, bevy_rtin::MeshOptions, prelude::*};
use bevy_rapier3d::{math::Vect, prelude::*};
use std::path::PathBuf;

pub struct WorldPlugin {
    pub(crate) terrain_path: PathBuf,
}

#[derive(Component, Debug)]
pub struct Leash(pub f32);

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (make_spawn_floor(self.terrain_path.clone()), spawn_light),
        );
        app.add_systems(Update, wireframe_control);
        app.add_systems(
            Update,
            handle_prespawning_inputs.run_if(in_state(GameState::Prespawn)),
        );
        app.add_systems(FixedUpdate, leash_system);
    }
}
fn wireframe_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<WireframeConfig>,
    mut rapier_wireframes: ResMut<DebugRenderContext>,
) {
    // Toggle showing a wireframe on all meshes
    if keyboard_input.just_pressed(KeyCode::F4) {
        info!(
            "Toggling wireframes {}",
            if config.global { "off" } else { "on" }
        );
        config.global = !config.global;
        rapier_wireframes.enabled = !rapier_wireframes.enabled;
    }
}
fn handle_prespawning_inputs(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut mouse: Res<ButtonInput<MouseButton>>,
    mut config: ResMut<WireframeConfig>,
    mut rapier_wireframes: ResMut<DebugRenderContext>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        next_state.set(GameState::Spawning);
    }
}
fn make_spawn_floor(
    terrain_path: PathBuf,
) -> impl FnMut(Commands, ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>) {
    move |mut commands, mut meshes, mut materials| {
        let (shaded, shaded_mesh_data) =
            bevy_rtin::load_mesh(&terrain_path, MeshOptions::default()).unwrap();
        info!("Spawning terrain mesh from {:?}", terrain_path);
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
                    transform: Transform::from_scale(Vec3::new(1., 150.0, 1.0)),
                    ..default()
                },
                RigidBody::Fixed,
                Name::new("shaded_floor"),
            ))
            .with_children(|p| {
                p.spawn((
                    Name::new("terrain_collider"),
                    collider,
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
    let light_grid_size = 3;
    let interval = grid_size / light_grid_size as f32;

    info!(
        "Spawning {}x{} point lights",
        light_grid_size, light_grid_size
    );

    for x in 0..light_grid_size {
        for z in 0..light_grid_size {
            let transform = Transform::from_xyz(x as f32 * interval, 150., z as f32 * interval);
            commands.spawn((
                PointLightBundle {
                    point_light: PointLight {
                        color: Color::srgba(1.0, 1.0, 1.0, 1.000),
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

fn leash_system(
    mut commands: Commands,
    mut n: Local<usize>,
    leashes: Query<(Entity, &GlobalTransform, &Leash)>,
) {
    *n = (*n + 1) % 10;
    if *n == 0 {
        for (entity, global_transform, leash) in &leashes {
            if global_transform.translation().length() > leash.0 {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
