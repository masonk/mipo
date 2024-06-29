use bevy::{
    math::vec3,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};

use crate::player::EnablePlayerControl;
use bevy_lunex::prelude::*;
use smooth_bevy_cameras::controllers::unreal::{UnrealCameraBundle, UnrealCameraController};
pub struct CameraPlugin;

#[derive(Debug, Clone, Component)]
pub(crate) struct Flycam;

#[derive(Debug, Clone, Component)]
pub(crate) struct FirstPersonCam;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, spawn_camera);
        app.add_systems(Update, handle_input);
    }
}

fn spawn_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Create a texture resource that our 3D camera will render to
    let size = Extent3d {
        width: 1920,
        height: 1080,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    // Initiate the image
    image.resize(size);

    // Add our texture to asset server and get a handle
    let image_handle = images.add(image);
    commands.spawn((MainUi, camera2d()));

    commands
        .spawn((
            // This makes the UI entity able to receive camera data
            MovableByCamera,
            // This is our UI system
            UiTreeBundle::<MainUi>::from(UiTree::new("Hello UI!")),
        ))
        .with_children(|ui| {
            let root = UiLink::<MainUi>::path("Root");

            ui.spawn((
                root.add("Camera3d"),
                UiLayout::solid()
                    .size((1920.0, 1080.0))
                    .scaling(Scaling::Fill)
                    .pack::<Base>(),
                UiImage2dBundle::from(image_handle.clone()),
            ));
        });

    // Spawn 3D camera view which will become an image texture in the main 2d scene
    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                is_active: true,
                clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                // target: image_handle.into(),
                ..default()
            },
            ..default()
        })
        .insert(Flycam)
        .insert(UnrealCameraBundle::new(
            flycam_controller(),
            vec3(-154.44, 204.027, -111.268),
            vec3(150., 20.0, 150.0),
            Vec3::Y,
        ));
}

pub fn camera2d() -> impl Bundle {
    use bevy::core_pipeline::bloom::BloomSettings;

    (
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
    )
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
