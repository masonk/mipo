use bevy::{
    input::{mouse::MouseMotion, InputSystem},
    log::prelude::*,
    prelude::*,
};
use glam::vec3;

pub struct SpinnerUiPlugin;

impl Plugin for SpinnerUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup));
    }
}

#[derive(Component)]
struct Disc {
    angle: f32,
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: asset_server.load("torus1.stl"),
            material: materials.add(Color::srgb(0.9, 0.4, 0.3)),
            transform: Transform {
                translation: vec3(100.0, 75.0, 0.0),
                rotation: Quat::from_rotation_x(f32::to_radians(90.0)),
                scale: vec3(0.5, 0.5, 0.5),
            },
            ..Default::default()
        },
        Disc { angle: 0.0 },
        Name::new("Spinner"),
    ));
}
