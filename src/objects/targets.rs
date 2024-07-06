use bevy::{
    input::common_conditions::*,
    input::mouse::{MouseButton, MouseButtonInput},
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    window::PrimaryWindow,
};
use bevy_rapier3d::prelude::*;

use crate::palette::Palette;
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
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });
    let cube = meshes.add(Sphere::new(1.0));
    let collider = Collider::ball(1.0);
    let mut rng = rand::thread_rng();

    for (entity_id, targets) in &query {
        info!("Detected Targets addition. Spawning...");
        let mut position = targets.extent.clone();
        position.y += 30.0;
        let boundary_cube = meshes.add(Cuboid::from_size(targets.extent));

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

// fn despawn_targets(
//     mut commands: Commands,
//     query: Query<Entity, Added<Targets>>,
//     // window: Query<&Window>,
//     asset_server: Res<AssetServer>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
// }

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn rotate(mut query: Query<(&Target, &mut Transform), With<Target>>, time: Res<Time>) {
    for (target, mut transform) in &mut query {
        transform.rotate_x(time.delta_seconds() / target.rotation_velocity.x);
        transform.rotate_y(time.delta_seconds() / target.rotation_velocity.y);
        transform.rotate_z(time.delta_seconds() / target.rotation_velocity.z);
    }
}

pub fn cast_ray(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    rapier_context: Res<RapierContext>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut material_query: Query<&mut Handle<StandardMaterial>>,
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

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // We will color in red the colliders hovered by the mouse.
    for (camera, camera_transform) in &cameras {
        // First, compute a ray from the mouse position.
        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };

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
