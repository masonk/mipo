use crate::palette::Palette;
use crate::player::EnablePlayerControl;
use bevy::{prelude::*, sprite::Anchor, window::CursorGrabMode, window::PrimaryWindow};
use bevy_lunex::prelude::*;
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
            BloomSettings::OLD_SCHOOL,
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
                // Cursor2d::new(),
                // .native_cursor(false)
                // .register_cursor(CursorIcon::Default, 0, (14.0, 14.0))
                // .register_cursor(CursorIcon::Pointer, 1, (10.0, 12.0))
                // .register_cursor(CursorIcon::Grab, 2, (40.0, 40.0)),
                // Add texture atlas to the cursor
                SpriteBundle {
                    texture: crosshairs,
                    transform: Transform {
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
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut fly_cam: Query<(Entity, &mut Camera), (With<Flycam>, Without<FirstPersonCam>)>,
    mut fps_cam: Query<&mut Camera, With<FirstPersonCam>>,
) {
    let mut window = match windows.get_single_mut() {
        Ok(w) => w,
        Err(e) => {
            return warn!("Couldn't find the PrimaryWindow for disabling cursor/enabling recticle")
        }
    };

    if keyboard.just_pressed(KeyCode::F3) {
        let (fly_cam_entity_id, mut fly_cam) = match fly_cam.get_single_mut() {
            Ok((fly_cam_entity_id, fly_cam)) => (fly_cam_entity_id, fly_cam),
            Err(e) => return warn!("Could not find Flycam, {e}"),
        };

        let mut fps_cam = match fps_cam.get_single_mut() {
            Ok(fps_cam) => fps_cam,
            Err(e) => return warn!("Couldn't get the fps cam: {e}"),
        };

        if fly_cam.is_active {
            info!("Setting fps_cam to active");
            commands
                .entity(fly_cam_entity_id)
                .remove::<UnrealCameraController>();
            enable.0 = true;
            fly_cam.is_active = false;
            fps_cam.is_active = true;
            window.cursor.visible = false;
        } else {
            info!("Setting fly_cam to active");
            enable.0 = false;
            fly_cam.is_active = true;
            fps_cam.is_active = false;
            commands
                .entity(fly_cam_entity_id)
                .insert(flycam_controller());
            window.cursor.visible = true;
            window.cursor.grab_mode = CursorGrabMode::Confined;
        }
    }
}
