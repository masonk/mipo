use bevy::prelude::*;

use smooth_bevy_cameras::controllers::unreal::{UnrealCameraBundle, UnrealCameraController};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle::default())
        .insert(UnrealCameraBundle::new(
            UnrealCameraController {
                keyboard_mvmt_sensitivity: 100.0,
                ..default()
            },
            Vec3::new(-154.44, 204.027, -111.268),
            Vec3::new(150., 20.0, 150.0),
            Vec3::Y,
        ));
}
