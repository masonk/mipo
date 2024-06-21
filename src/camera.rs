use bevy::prelude::*;
use bevy_third_person_camera::{
    camera::{CameraGamepadSettings, Zoom},
    ThirdPersonCamera,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    let camera = (
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ThirdPersonCamera {
            aim_enabled: true,
            aim_speed: 3.0, // default
            aim_zoom: 0.7,  // default
            offset_enabled: true,
            offset_toggle_enabled: true,
            gamepad_settings: CameraGamepadSettings { ..default() },
            zoom_enabled: true,           // default
            zoom: Zoom::new(1.5, 20000.), // default
            ..default()
        },
    );
    commands.spawn(camera);
}
