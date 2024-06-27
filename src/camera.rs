use bevy::math::vec3;
use bevy::prelude::*;

use crate::player::EnablePlayerControl;
use smooth_bevy_cameras::controllers::unreal::{UnrealCameraBundle, UnrealCameraController};
pub struct CameraPlugin;

#[derive(Debug, Clone, Component)]
pub(crate) struct Flycam;

#[derive(Debug, Clone, Component)]
pub(crate) struct FirstPersonCam;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(Update, handle_input);
    }
}

fn spawn_camera(mut commands: Commands) {
    let unreal = UnrealCameraBundle::new(
        flycam_controller(),
        vec3(-154.44, 204.027, -111.268),
        vec3(150., 20.0, 150.0),
        Vec3::Y,
    );

    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                is_active: false,
                ..default()
            },
            ..default()
        })
        .insert(Flycam)
        .insert(unreal);
}

fn flycam_controller() -> UnrealCameraController {
    UnrealCameraController {
        keyboard_mvmt_sensitivity: 100.0,
        ..default()
    }
}
fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut enable: ResMut<EnablePlayerControl>,
    mut commands: Commands,
    mut fly_cam: Query<(Entity, &mut Camera), Without<FirstPersonCam>>,
    mut fps_cam: Query<&mut Camera, With<FirstPersonCam>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        let (fly_cam_entity_id, mut fly_cam) = fly_cam.single_mut();
        let mut fps_cam = fps_cam.single_mut();

        if fly_cam.is_active {
            info!("Setting fps_cam to active");
            commands
                .entity(fly_cam_entity_id)
                .remove::<UnrealCameraController>();
            enable.0 = true;
            fly_cam.is_active = false;
            fps_cam.is_active = true;
        } else {
            info!("Setting fly_cam to active");
            enable.0 = false;
            fly_cam.is_active = true;
            fps_cam.is_active = false;
            commands
                .entity(fly_cam_entity_id)
                .insert(flycam_controller());
        }
    }
}
