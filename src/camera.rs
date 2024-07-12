use crate::palette::Palette;
use bevy::{
    prelude::*,
    sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle, Sprite},
    window::CursorGrabMode,
    window::{PrimaryWindow, WindowResized},
};
use bevy_lunex::prelude::*;
use smooth_bevy_cameras::controllers::unreal::{UnrealCameraBundle, UnrealCameraController};
pub struct CameraPlugin;

use crate::GameState;

#[derive(Debug, Clone, Component)]
pub(crate) struct Flycam;

#[derive(Debug, Clone, Component)]
pub(crate) struct FirstPersonCam;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(Update, handle_input);
        app.add_systems(OnEnter(GameState::InGame), enter_in_game);
        app.add_systems(OnEnter(GameState::DevMode), enter_dev_mode);
    }
}

fn spawn_camera(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
) {
    use crate::asset_cache::AssetCache;
    use bevy::core_pipeline::bloom::BloomSettings;
    let crosshairs: Handle<Image> = asset_server.load(AssetCache::CROSSHAIRS_SHEET);
    let texture_atlas_layout = atlas_layout.add(TextureAtlasLayout::from_grid(
        UVec2::splat(128),
        20,
        10,
        Some(UVec2::splat(10)),
        None,
    ));

    commands
        .spawn((
            MainUi,
            Camera2dBundle {
                transform: Transform::from_xyz(0.0, 0.0, 1000.0),
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                //tonemapping: Tonemapping::None,
                ..default()
            },
            BloomSettings::NATURAL,
            InheritedVisibility::default(),
            /*VfxWiggleCamera {
                sinusoid: vec![
                    Sine {
                        speed: 0.005,
                        amplitude: 0.003,
                        degree: 0.0,
                    }
                ]
            }*/
        ))
        .with_children(|camera| {
            camera.spawn((
                SpriteBundle {
                    texture: crosshairs,
                    transform: Transform {
                        translation: Vec3::new(0., 0., 500.),
                        scale: Vec3::new(0.45, 0.45, 1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        color: Color::BLACK,
                        anchor: Anchor::Center,
                        ..default()
                    },
                    ..default()
                },
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 51,
                },
            ));
        });
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        if *state.get() == GameState::DevMode {
            info!("Setting GameState to InGame");
            next_state.set(GameState::InGame);
        } else {
            info!("Setting GameState to DevMode");
            next_state.set(GameState::DevMode);
        }
    }
}

fn enter_in_game(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut fps_cam: Query<&mut Camera, With<FirstPersonCam>>,
    mut fly_cam: Query<
        (&mut UnrealCameraController, &mut Camera),
        (With<Flycam>, Without<FirstPersonCam>),
    >,
) {
    let mut window = match windows.get_single_mut() {
        Ok(w) => w,
        Err(e) => {
            return warn!("Couldn't find the PrimaryWindow for disabling cursor/enabling recticle")
        }
    };
    info!("Hiding cursor in FPS mode");
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;
    let center_x = window.width() / 2.;
    let center_y = window.height() / 2.;
    window.set_cursor_position(Some((center_x, center_y).into()));
    if let Ok(mut fps_cam) = fps_cam.get_single_mut() {
        fps_cam.is_active = true;
    } else {
        warn!("Unable to find FirstPersonCam to activate it.");
    }
    if let Ok((mut controller, mut fly_cam)) = fly_cam.get_single_mut() {
        controller.enabled = false;
        fly_cam.is_active = false;
    } else {
        warn!("Unable to find Flycam to deactivate it.");
    }
}

fn enter_dev_mode(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut fps_cam: Query<&mut Camera, With<FirstPersonCam>>,
    mut fly_cam: Query<
        (&mut UnrealCameraController, &mut Camera),
        (With<Flycam>, Without<FirstPersonCam>),
    >,
) {
    let mut window = match windows.get_single_mut() {
        Ok(w) => w,
        Err(e) => {
            return warn!("Couldn't find the PrimaryWindow for disabling cursor/enabling recticle")
        }
    };
    info!("Showing cursor in dev mode.");
    window.cursor.visible = true;
    window.cursor.grab_mode = CursorGrabMode::None;
    if let Ok(mut fps_cam) = fps_cam.get_single_mut() {
        fps_cam.is_active = false;
    } else {
        warn!("Unable to find FirstPersonCam to deactivate it.");
    }
    if let Ok((mut controller, mut fly_cam)) = fly_cam.get_single_mut() {
        controller.enabled = true;
        fly_cam.is_active = true;
    } else {
        warn!("Unable to find Flycam to activate it.");
    }
}
