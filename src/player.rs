use crate::camera::FirstPersonCam;
use bevy::{
    input::{mouse::MouseMotion, InputSystem},
    log::prelude::*,
    prelude::*,
};
use bevy_rapier3d::control::KinematicCharacterController;
use bevy_rapier3d::prelude::*;

const MOUSE_SENSITIVITY: f32 = 0.3;
const GROUND_TIMER: f32 = 0.5;
const MOVEMENT_SPEED: f32 = 8.0;
const JUMP_SPEED: f32 = 20.0;
const GRAVITY: f32 = -9.81;

#[derive(Component)]
struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementInput>()
            .init_resource::<LookInput>()
            .init_resource::<EnablePlayerControl>()
            .add_systems(Startup, spawn_player)
            .add_systems(PreUpdate, handle_input.after(InputSystem))
            .add_systems(Update, player_look)
            .add_systems(FixedUpdate, player_movement);
    }
}

#[derive(Debug, Resource)]
pub struct EnablePlayerControl(pub bool);
impl Default for EnablePlayerControl {
    fn default() -> Self {
        EnablePlayerControl(true)
    }
}

/// Keyboard input vector
#[derive(Default, Resource, Deref, DerefMut)]
struct MovementInput(Vec3);

/// Mouse input vector
#[derive(Default, Resource, Deref, DerefMut)]
struct LookInput(Vec2); // Degrees that the user has turned since last update.

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    enable: Res<EnablePlayerControl>,
    mut movement: ResMut<MovementInput>,
    mut look: ResMut<LookInput>,
    mut mouse_events: EventReader<MouseMotion>,
) {
    // if !enable.0 {
    //     return;
    // }
    if keyboard.pressed(KeyCode::KeyW) {
        movement.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        movement.z += 1.0
    }
    if keyboard.pressed(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        movement.x += 1.0
    }
    **movement = movement.normalize_or_zero();
    if keyboard.pressed(KeyCode::ShiftLeft) {
        **movement *= 2.0;
    }
    if keyboard.pressed(KeyCode::Space) {
        movement.y = 1.0;
    }

    for event in mouse_events.read() {
        look.x -= event.delta.x * MOUSE_SENSITIVITY;
        look.y -= event.delta.y * MOUSE_SENSITIVITY;
        look.y = look.y.clamp(-89.9, 89.9); // Limit pitch
    }
}

fn player_movement(
    time: Res<Time>,
    enable: Res<EnablePlayerControl>,
    mut input: ResMut<MovementInput>,
    mut player: Query<(
        &mut Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    mut vertical_movement: Local<f32>,
    mut grounded_timer: Local<f32>,
) {
    if !enable.0 {
        return;
    }
    let Ok((transform, mut controller, output)) = player.get_single_mut() else {
        return;
    };
    let delta_time = time.delta_seconds();
    // Retrieve input
    let mut movement = Vec3::new(input.x, 0.0, input.z) * MOVEMENT_SPEED;
    let jump_speed = input.y * JUMP_SPEED;
    // Clear input
    **input = Vec3::ZERO;
    // Check physics ground check
    if output.map(|o| o.grounded).unwrap_or(false) {
        *grounded_timer = GROUND_TIMER;
        *vertical_movement = 0.0;
    }
    // If we are grounded we can jump
    if *grounded_timer > 0.0 {
        *grounded_timer -= delta_time;
        // If we jump we clear the grounded tolerance
        if jump_speed > 0.0 {
            *vertical_movement = jump_speed;
            *grounded_timer = 0.0;
        }
    }
    movement.y = *vertical_movement;
    *vertical_movement += GRAVITY * delta_time * controller.custom_mass.unwrap_or(1.0);
    controller.translation = Some(transform.rotation * (movement * delta_time));
}

fn player_look(
    mut player: Query<
        &mut Transform,
        (With<KinematicCharacterController>, Without<FirstPersonCam>),
    >,
    mut camera: Query<&mut Transform, With<FirstPersonCam>>,
    look: ResMut<LookInput>,
) {
    if look.x == 0.0 && look.y == 0.0 {
        return;
    }
    let mut player_transform = match player.get_single_mut() {
        Ok(transform) => transform,
        Err(e) => return warn!("Failed to look up player transform: {e}"),
    };

    // Rotating the player in the xz plane also rotates the player's child camera
    player_transform.rotation = Quat::from_axis_angle(Vec3::Y, look.x.to_radians());

    let mut camera_transform = match camera.get_single_mut() {
        Ok(t) => t,
        Err(e) => return warn!("Failed to look up player camera transformer: {e}"),
    };

    // we additionally want to rotate the player camera in the y direction but not rotate the player's body
    camera_transform.rotation = Quat::from_axis_angle(Vec3::X, look.y.to_radians());
    // pull the camera out of the player's body by 1.0 in the direction the player is facing.
    camera_transform.translation = camera_transform.forward() * 1.0;
    camera_transform.translation.y = 1.0;
}

fn spawn_player(
    mut commands: Commands,
    assets: Res<AssetServer>, // mut meshes: ResMut<Assets<Mesh>>,
                              // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let flashlight = (
        SpotLightBundle {
            spot_light: SpotLight {
                shadows_enabled: true,
                color: Color::srgba(0.927, 0.855, 0.507, 1.000),
                intensity: 100000.0,
                range: 9.0,
                outer_angle: 0.5,
                inner_angle: 0.4,
                ..default()
            },
            transform: Transform::from_xyz(0., 0.5, 0.),
            ..default()
        },
        Name::new("flashlight"),
    );

    let player = (
        Name::new("player"),
        SceneBundle {
            scene: assets.load("Player.gltf#Scene0"),
            transform: Transform::from_xyz(102.173, 46.668, 54.987),
            ..default()
        },
        RigidBody::KinematicPositionBased,
        Collider::cuboid(0.3, 1.0, 0.3),
        KinematicCharacterController {
            custom_mass: Some(5.0),
            up: Vec3::Y,
            offset: CharacterLength::Absolute(0.01),
            slide: true,
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Relative(0.3),
                min_width: CharacterLength::Relative(0.5),
                include_dynamic_bodies: false,
            }),
            // Don’t allow climbing slopes larger than 45 degrees.
            max_slope_climb_angle: 45.0_f32.to_radians(),
            // Automatically slide down on slopes smaller than 30 degrees.
            min_slope_slide_angle: 30.0_f32.to_radians(),
            apply_impulse_to_dynamic_bodies: true,
            snap_to_ground: None,
            ..default()
        },
        Player,
    );

    commands.spawn(player).with_children(|b| {
        b.spawn(flashlight);
        b.spawn((
            Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: 1.0,
                    ..default()
                }),
                ..Default::default()
            },
            Name::new("FirstPersonCamera"),
            FirstPersonCam,
        ));
    });
}
