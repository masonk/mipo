use std::{ops::DerefMut, time::Duration};

use crate::{
    asset_cache::AssetCache,
    camera::FirstPersonCam,
    hitpoints::{Hp, HpRegen},
    items::{FireballAbility, Platform},
    mana::{Mana, ManaRegen},
    prelude::*,
    GameState,
};
use bevy::{
    core_pipeline::Skybox,
    ecs::component::StorageType,
    input::{mouse::MouseMotion, InputSystem},
    log::prelude::*,
    prelude::*,
};
use bevy_rapier3d::{control::KinematicCharacterController, prelude::*};

const MOUSE_SENSITIVITY: f32 = 0.3;
// if the user has been grounded within x seconds and hasn't jumped within that time, he's grounded.
const GROUND_TIMER: f32 = 0.5;
// If the player has been on a platform within this amount of time and has not jumped, we impart the platform's
// linvel to the player.
const ON_PLATFORM_TIMER: f32 = 0.5;
const MOVEMENT_SPEED: f32 = 8.0;
const JUMP_SPEED: f32 = 40.0;
const GRAVITY: f32 = -9.81;
const AIR_JUMPS: u32 = 10;

#[derive(Default)]
pub struct Player;

impl Component for Player {
    const STORAGE_TYPE: bevy::ecs::component::StorageType = StorageType::Table;
    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_remove(|mut world, _player_entity, _component_id| {
            let mut state: Mut<NextState<GameState>> = world.resource_mut();
            info!("Player despawned, setting GameState to Prespawn.");
            state.set(GameState::Prespawn);
        });
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        info!("Installing PlayerPlugin");
        app.init_resource::<MovementInput>()
            .init_resource::<LookInput>()
            .add_systems(OnEnter(GameState::Spawning), spawn_player)
            .add_systems(PreUpdate, handle_input.after(InputSystem))
            .add_systems(Update, player_look)
            .add_systems(FixedUpdate, player_movement);
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
    mut movement: ResMut<MovementInput>,
    mut look: ResMut<LookInput>,
    mut mouse_events: EventReader<MouseMotion>,
    state: Res<State<GameState>>,
) {
    if *state.get() != GameState::InGame {
        return;
    }
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
    if keyboard.just_pressed(KeyCode::Space) {
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
    mut input: ResMut<MovementInput>,
    mut player: Query<
        (
            Entity,
            &mut Transform,
            &GlobalTransform,
            &mut KinematicCharacterController,
            Option<&KinematicCharacterControllerOutput>,
        ),
        With<Player>,
    >,
    mut vertical_movement: Local<f32>,
    mut grounded_timer: Local<f32>,
    mut grounded_platform_timer: Local<f32>,
    mut grounded_platform_linvel: Local<Vec3>,
    mut air_jumps_left: Local<u32>,
    platforms: Query<(Entity, &GlobalTransform, &Platform), Without<Player>>,
    rapier_context: Res<RapierContext>,
) {
    let Ok((player_entity, player_transform, player_global_transform, mut controller, output)) =
        player.get_single_mut()
    else {
        return;
    };
    let delta_time = time.delta_seconds();
    // Retrieve input
    let mut movement = Vec3::new(input.x, 0.0, input.z) * MOVEMENT_SPEED;
    // Is the player jumping?
    let jump_speed = input.y * JUMP_SPEED;
    // Clear input
    **input = Vec3::ZERO;
    // Check physics ground check
    if output.map(|o| o.grounded).unwrap_or(false) {
        *grounded_timer = GROUND_TIMER;
        *air_jumps_left = AIR_JUMPS;
        *vertical_movement = 0.0;
    }

    // If we are grounded we can jump
    if *grounded_timer > 0.0 {
        *grounded_timer -= delta_time;
        // If we jump we clear the grounded tolerance
        if jump_speed > 0.0 {
            *vertical_movement = jump_speed;
            // Unground me.
            *grounded_timer = 0.0;
        }
    } else {
        if jump_speed > 0.0 {
            if *air_jumps_left > 0 {
                *air_jumps_left -= 1;
                *vertical_movement += jump_speed;
            }
        }
    }
    movement.y = *vertical_movement;
    *vertical_movement += GRAVITY * delta_time * controller.custom_mass.unwrap_or(1.0);
    let mut translation = player_transform.rotation * movement;

    for (platform_entity, platform_global_transform, platform) in &platforms {
        if let Some(_contact_pair) = rapier_context.contact_pair(player_entity, platform_entity) {
            // TODO: Figure transform the platform linvel (in world coordinates) to local player coordinates.'
            let global_platform_position = platform_global_transform.translation();
            let global_player_position = player_global_transform.translation();
            if global_player_position.y > global_platform_position.y + 3.0 {
                // player is standing on the platform.
                *grounded_platform_timer = ON_PLATFORM_TIMER;
                *grounded_platform_linvel = platform.linvel;
                // Note:  has_any_active_contacts() always returns false, because the kinematic character controller keeps the player very slightly floating.
                // we are sort of faking this by just checking whether the player is very close to the platform.
            }
        }
    }
    // Rapier is not reliably returning a collision between the player and the platform on every frame
    // This was causing slippage, where some frames the player does not get the platform's linvel
    // We add a timeout period so that if the player has been on a platform within ON_PLATFORM_TIMER seconds,
    // We add the linvel of the platform that the player was most recently on to the player's linvel.
    if *grounded_platform_timer > 0.0 {
        *grounded_platform_timer -= delta_time;
        translation += *grounded_platform_linvel;
        if jump_speed > 0.0 {
            // If the player jumped in this frame, then immediately set him to off the platform.
            *grounded_platform_timer = 0.0;
        }
    }
    controller.translation = Some(translation * delta_time);
}

fn player_look(
    mut player: Query<
        &mut Transform,
        (
            With<KinematicCharacterController>,
            With<Player>,
            Without<FirstPersonCam>,
        ),
    >,
    mut camera: Query<&mut Transform, With<FirstPersonCam>>,
    look: ResMut<LookInput>,
) {
    if look.x == 0.0 && look.y == 0.0 {
        return;
    }
    let mut player_transform = match player.get_single_mut() {
        Ok(transform) => transform,
        Err(e) => return,
    };

    // Rotating the player in the xz plane also rotates the player's child camera
    player_transform.rotation = Quat::from_axis_angle(Vec3::Y, look.x.to_radians());

    let mut camera_transform = match camera.get_single_mut() {
        Ok(t) => t,
        Err(e) => return warn!("Failed to look up player camera transform: {e}"),
    };
    // we additionally want to rotate the player camera in the y direction but not rotate the player's body
    camera_transform.rotation = Quat::from_axis_angle(Vec3::X, look.y.to_radians());
}

fn spawn_player(
    mut commands: Commands,
    assets: Res<AssetServer>, // mut meshes: ResMut<Assets<Mesh>>,
    cache: Res<AssetCache>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<GameState>>,
    game_world: Res<GameWorldImage>,
) {
    info!("Spawning Player");
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
    let mut fireball_timer =
        Timer::new(std::time::Duration::from_millis(100), TimerMode::Repeating);

    // Player should start with it ready.
    fireball_timer.tick(Duration::from_millis(1000));

    let player = (
        Name::new("player"),
        SceneBundle {
            scene: assets.load("Player.gltf#Scene0"),
            transform: Transform::from_xyz(102.173, 250., 54.987)
                .looking_at(Vec3::new(0., -1., -1.), Vec3::Y),
            ..default()
        },
        Leash(1000.),
        RigidBody::KinematicPositionBased,
        Collider::cuboid(0.3, 1.0, 0.3),
        ActiveEvents::COLLISION_EVENTS, // Make sure that we always solve for player contacts.
        KinematicCharacterController {
            custom_mass: Some(5.0),
            up: Vec3::Y,
            offset: CharacterLength::Absolute(0.0001),
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
            snap_to_ground: Some(CharacterLength::Absolute(0.2)),
            ..default()
        },
        Player,
        FireballAbility {
            mana_cost: 5,
            cooldown_timer: fireball_timer,
            projectile_speed: 70.,
            projectile_radius: 0.3,
            gravity: 1.,
            damping: 1.,
            ..default()
        },
        Hp {
            current: 100,
            max: 100,
        },
        HpRegen {
            tick_timer: Timer::new(Duration::from_millis(1000), TimerMode::Repeating),
            regen_per_tick: 5,
        },
        Mana {
            current: 100,
            max: 100,
        },
        ManaRegen {
            regen_mana_timer: Timer::new(Duration::from_millis(1000), TimerMode::Repeating),
            regen_per_tick: 5,
        },
    );

    commands.spawn(player).with_children(|b| {
        b.spawn(flashlight);
        b.spawn((
            // StateScoped(GameState::InGame),
            Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: 50.0_f32.to_radians(),
                    ..default()
                }),
                camera: Camera {
                    is_active: true,
                    order: 10,
                    target: game_world.0.clone().into(),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.7, -1.0),
                ..default()
            },
            Skybox {
                image: cache.skybox.clone(),
                brightness: 1000.,
            },
            Name::new("FirstPersonCamera"),
            FirstPersonCam,
        ));
        // b.spawn((
        //     Camera3dBundle {
        //         projection: Projection::Perspective(PerspectiveProjection {
        //             fov: 1.0,
        //             ..default()
        //         }),
        //         transform: Transform::from_xyz(0.0, 0.7, -1.0),
        //         ..default()
        //     },
        //     Name::new("FirstPersonCamera"),
        //     FirstPersonCam,
        // ));
    });
    next_state.set(GameState::InGame)
}
