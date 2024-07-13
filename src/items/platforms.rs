use crate::asset_cache::AssetCache;
use bevy::{prelude::*, time::Timer};
use bevy_rapier3d::{na::Quaternion, prelude::*};
use rand::seq::SliceRandom;
use std::time::Duration; // 0.7.2

use rand::{self, thread_rng, Rng};

pub struct PlatformsPlugin;

#[derive(Default, Component)]
struct PlatformSpawner {
    timer: Timer,
}

#[derive(Resource)]
struct PlatformMeshes {
    meshes: Vec<Handle<Mesh>>,
}

#[derive(Default, Component)]
pub struct Platform {
    pub linvel: Vec3,
}

impl Plugin for PlatformsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup_platforms);
        app.add_systems(FixedUpdate, update_platforms);
    }
}

fn startup_platforms(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    // let meshes = vec![
    //     meshes.add(Extrusion::new(Rectangle::default(), 1.)),
    //     meshes.add(Extrusion::new(Capsule2d::default(), 1.)),
    //     meshes.add(Extrusion::new(Annulus::default(), 1.)),
    //     meshes.add(Extrusion::new(Circle::default(), 1.)),
    //     meshes.add(Extrusion::new(Ellipse::default(), 1.)),
    //     meshes.add(Extrusion::new(RegularPolygon::default(), 1.)),
    //     meshes.add(Extrusion::new(Triangle2d::default(), 1.)),
    // ];
    let meshes = vec![meshes.add(Extrusion::new(Rectangle::new(100., 100.), 5.))];
    commands.insert_resource(PlatformMeshes { meshes });
    commands.spawn(
        (PlatformSpawner {
            timer: Timer::from_seconds(0.0, TimerMode::Repeating),
        }),
    );
}
fn update_platforms(
    mut commands: Commands,
    time: Res<Time>,
    mut spawners: Query<&mut PlatformSpawner>,
    mut platforms: Query<(&mut Transform, &Platform)>,
    meshes: Res<Assets<Mesh>>,
    asset_cache: Res<AssetCache>,
    platform_meshes: Res<PlatformMeshes>,
) {
    let delta_time = time.delta_seconds();
    for mut spawner in spawners.iter_mut() {
        if spawner.timer.tick(time.delta()).finished() {
            let picked_mesh = platform_meshes
                .meshes
                .choose(&mut rand::thread_rng())
                .unwrap();
            let mut transform = Transform::from_translation(Vec3::new(
                thread_rng().gen_range(-75.0..-25.0),
                thread_rng().gen_range(25.0..75.0),
                thread_rng().gen_range(-75.0..-25.0),
            ));
            transform.rotate_x(90.0f32.to_radians());
            commands
                .spawn((
                    PbrBundle {
                        mesh: picked_mesh.clone(),
                        material: asset_cache.debug_material.clone(),
                        transform,
                        ..default()
                    },
                    Platform {
                        // linvel: Vec3::splat(0.),
                        linvel: Vec3::new(
                            thread_rng().gen_range(0.33..0.66),
                            0.,
                            thread_rng().gen_range(0.33..0.66),
                        )
                        .normalize()
                            * thread_rng().gen_range(3.0..10.0),
                    },
                ))
                .insert((
                    RigidBody::KinematicPositionBased,
                    Collider::from_bevy_mesh(
                        meshes
                            .get(picked_mesh)
                            .expect("Couldn't get a mesh entity to spawn a collider."),
                        &ComputedColliderShape::TriMesh,
                    )
                    .unwrap(),
                ));
            spawner
                .timer
                .set_duration(Duration::from_millis(thread_rng().gen_range(12000..12001)));
        }
    }
    for (mut transform, platform) in platforms.iter_mut() {
        transform.translation += platform.linvel * delta_time;
    }
}
