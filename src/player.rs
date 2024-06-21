use bevy::prelude::*;
use bevy_third_person_camera::{controller::ThirdPersonController, ThirdPersonCameraTarget};

#[derive(Component)]
struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);
        // app.add_systems(Update, player_movement);
    }
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
                color: Color::rgba(0.927, 0.855, 0.507, 1.000),
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
            transform: Transform::from_xyz(128.159, 0.5, 152.334),
            ..default()
        },
        Player,
        ThirdPersonCameraTarget,
        ThirdPersonController {
            speed: 5.,
            sprint_enabled: true,
            sprint_speed: 7.5,
            ..default()
        }, // add third person controller
    );

    commands.spawn(player).with_children(|parent| {
        parent.spawn(flashlight);
    });
}

// fn player_movement(
//     keys: Res<ButtonInput<KeyCode>>,
//     time: Res<Time>,
//     mut player_q: Query<(&mut Transform), With<Player>>,
//     cam_q: Query<&Transform, (With<Camera3d>, Without<Player>)>,
// ) {
//     for (mut player_transform, player_speed) in player_q.iter_mut() {
//         let cam = match cam_q.get_single() {
//             Ok(c) => c,
//             Err(e) => Err(format!("Error retrieving camera: {}", e)).unwrap(),
//         };
//         let mut direction = Vec3::ZERO;
//         // forward
//         if keys.pressed(KeyCode::KeyW) {
//             direction += *cam.forward()
//         }
//         if keys.pressed(KeyCode::KeyS) {
//             direction += *cam.back()
//         }
//         if keys.pressed(KeyCode::KeyA) {
//             direction += *cam.left()
//         }
//         if keys.pressed(KeyCode::KeyD) {
//             direction += *cam.right()
//         }
//         direction.y = 0.0;
//         let movement = direction.normalize_or_zero() * player_speed.0 * time.delta_seconds();
//         player_transform.translation += movement;

//         if direction.length_squared() > 0. {
//             player_transform.look_to(direction, Vec3::Y)
//         }
//     }
// }
