use bevy::{
    input::{common_conditions::*, mouse::MouseButton},
    prelude::*,
    render::{
        camera,
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    window::PrimaryWindow,
};
use bevy_rapier3d::prelude::*;

use crate::palette::Palette;
use crate::{asset_cache, camera::FirstPersonCam};
use rand::{self, Rng};

#[derive(Component)]
pub struct Targets {
    number: u32,
    name: String,
    extent: Vec3,
}

impl Default for Targets {
    fn default() -> Self {
        Targets {
            number: 5,
            name: "Targets".into(),
            extent: (50., 50., 50.).into(),
        }
    }
}

#[derive(Component, Default)]
pub struct Target {
    rotation_velocity: Vec3,
}

impl Target {
    fn from_rotations(rotation_velocity: Vec3) -> Self {
        Target { rotation_velocity }
    }
}

pub struct TargetsPlugin;

impl Plugin for TargetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, load_targets);
        app.add_systems(Update, cast_ray.run_if(input_pressed(MouseButton::Left)));

        // app.add_systems(PreUpdate, despawn_targets);
    }
}

fn load_targets(
    mut commands: Commands,
    query: Query<(Entity, &Targets), Added<Targets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_cache: Res<asset_cache::AssetCache>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_cache.debug_image.clone()),
        ..default()
    });
    let cube = meshes.add(Sphere::new(1.0));
    let collider = Collider::ball(1.0);
    let mut rng = rand::thread_rng();

    for (entity_id, targets) in &query {
        info!("Detected Targets addition. Spawning...");
        let mut position = targets.extent.clone();
        position.y += 30.0;
        let _boundary_cube = meshes.add(Cuboid::from_size(targets.extent));

        let mut entity = commands.entity(entity_id);
        let mut base_color = Palette::Blue.to_color();
        base_color.set_alpha(0.1);

        entity.insert((
            Name::new(targets.name.clone()),
            SpatialBundle::from_transform(Transform::from_translation(position)), // Transform::from_translation(position),
        ));
        // This spawns a transparent boundary cube to show the volume where targets can possibly spawn.
        // .insert(PbrBundle {
        //     transform: Transform::from_translation(position),
        //     mesh: boundary_cube,
        //     material: materials.add(StandardMaterial {
        //         base_color,
        //         alpha_mode: AlphaMode::Blend,
        //         ..default()
        //     }),
        //     ..default()
        // });
        entity.with_children(|children| {
            for i in 0..targets.number {
                let translation: Vec3 = (
                    rng.gen_range(-targets.extent.x / 2.0..targets.extent.x / 2.0),
                    rng.gen_range(-targets.extent.y / 2.0..targets.extent.y / 2.0),
                    rng.gen_range(-targets.extent.z / 2.0..targets.extent.z / 2.0),
                )
                    .into();
                children.spawn((
                    Target::from_rotations(translation.clone()),
                    Name::new(format!("target{i}")),
                    RigidBody::Dynamic,
                    GravityScale(0.0),
                    collider.clone(),
                    PbrBundle {
                        transform: Transform::from_translation(translation),
                        mesh: cube.clone(),
                        material: debug_material.clone(),
                        ..default()
                    },
                ));
            }
        });
    }
}

fn rotate(mut query: Query<(&Target, &mut Transform), With<Target>>, time: Res<Time>) {
    for (target, mut transform) in &mut query {
        transform.rotate_x(time.delta_seconds() / target.rotation_velocity.x);
        transform.rotate_y(time.delta_seconds() / target.rotation_velocity.y);
        transform.rotate_z(time.delta_seconds() / target.rotation_velocity.z);
    }
}

pub fn viewport_to_ndc(logical_viewport_size: Vec2, mut screen_coordinates: Vec2) -> Vec2 {
    // Flip the Y co-ordinate origin from the top to the bottom.
    screen_coordinates.y = logical_viewport_size.y - screen_coordinates.y;
    screen_coordinates * 2. / logical_viewport_size - Vec2::ONE
}

pub fn cast_ray(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    rapier_context: Res<RapierContext>,
    cameras: Query<(&Camera, &GlobalTransform, &Transform), With<FirstPersonCam>>,
    mut gizmos: Gizmos,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // keyboard: Res<ButtonInput<KeyCode>>,
    // enable: Res<EnablePlayerControl>,
    // mut movement: ResMut<MovementInput>,
    // mut look: ResMut<LookInput>,
    mut mouse: Res<ButtonInput<MouseButton>>,
) {
    let window = windows.single();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    // We will color in red the colliders hovered by the mouse.
    for (camera, camera_global_transform, camera_transform) in &cameras {
        // First, compute a ray from the mouse position.
        let Some(ray) = camera.viewport_to_world(camera_global_transform, cursor_position) else {
            warn!("no ray");
            return;
        };

        let logical_viewport_size = match camera.logical_viewport_size() {
            Some(size) => size,
            None => return warn!("no viewport"),
        };

        let mut _cursor_ndc = viewport_to_ndc(logical_viewport_size, cursor_position).extend(1.0);

        let _near = camera
            .ndc_to_world(camera_global_transform, camera_transform.translation)
            .unwrap();

        // gizmos.ray(near, ray.get_point(25.0), Palette::Red);

        // Because of the query filter, only colliders attached to a dynamic body
        // will get an event.
        let hit = rapier_context.cast_ray(
            ray.origin,
            ray.direction.into(),
            f32::MAX,
            true,
            QueryFilter::only_dynamic(),
        );

        if let Some((entity, _toi)) = hit {
            let highlight = materials.add(StandardMaterial {
                base_color: Palette::Red.to_color().into(),
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            commands.entity(entity).insert(highlight);
        }
    }
}
