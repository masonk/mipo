use bevy::render::view;
use bevy::sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle};
use bevy::{
    math::vec3,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    window::{PrimaryWindow, WindowResized},
};

use bevy_lunex::prelude::*;
use smooth_bevy_cameras::controllers::unreal::{UnrealCameraBundle, UnrealCameraController};

use crate::camera::Flycam;
use crate::palette::Palette;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct HudRoute;

pub struct HudRoutePlugin;

#[derive(Component)]
struct GameWorldImage(Handle<Image>);

impl Plugin for HudRoutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, build_route.before(UiSystems::Compute));
        app.add_systems(Update, update_gameworld_viewport);
    }
}

fn update_gameworld_viewport(
    mut images: ResMut<Assets<Image>>,
    mut resize_reader: EventReader<WindowResized>,
    mut query: Query<(&mut GameWorldImage, &mut Camera), With<Flycam>>,
) {
    for e in resize_reader.read() {
        debug!("Window resized. New extent: {}, {}", e.width, e.height);
        match query.get_single_mut() {
            Ok((game_world_image, camera)) => {
                // if let Some(_image) = images.get_mut(game_world_image.0.id()) {}
                if let Some(image) = images.get_mut(game_world_image.0.id()) {
                    let size = Extent3d {
                        width: e.width as u32,
                        height: e.height as u32,
                        ..default()
                    };
                    image.texture_descriptor.size = size;
                    image.resize(size);
                }

                // if let Some(ref mut viewport) = camera.viewport {
                //     viewport.physical_size.x = e.width as u32;
                //     viewport.physical_size.y = e.height as u32;
                // }

                // let projection = projection.as_mut();
                // match projection {
                //     Projection::Perspective(perspective) => {
                //         let prev = perspective.aspect_ratio;
                //         perspective.aspect_ratio = e.width / e.height;
                //         info!(
                //             "Updating 3d Game World aspect ration from {prev} to {}",
                //             perspective.aspect_ratio
                //         );
                //     }
                //     Projection::Orthographic(_ortho) => {}
                // }
            }
            _ => {
                warn!("No GameWorldImage to resize. Raycasting into the 3d game world probably doesn't work.")
            }
        };
    }
}

fn camera_image(width: u32, height: u32) -> Image {
    let size = Extent3d {
        width,
        height,
        ..default()
    };
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
    image.resize(size);
    image
}

fn build_route(
    mut commands: Commands,
    query: Query<Entity, Added<HudRoute>>,
    // window: Query<&Window>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();

    for route_entity in &query {
        // #======================#
        // #=== USER INTERFACE ===#

        info!("Spawning Hud Route");
        // let window = window.single();
        // Spawn the route
        commands
            .entity(route_entity)
            .insert(SpatialBundle::default())
            .with_children(|route| {
                let render_image =
                    asset_server.add(camera_image(window.width() as u32, window.height() as u32));

                info!("Spawning 3d camera");
                route
                    .spawn(Camera3dBundle {
                        projection: PerspectiveProjection {
                            // We must specify the FOV in radians.
                            // Rust can convert degrees to radians for us.
                            fov: 50.0_f32.to_radians(),
                            ..default()
                        }
                        .into(),
                        transform: Transform::from_translation(vec3(-154.44, 204.027, -111.268))
                            .looking_at(vec3(150., 20.0, 150.0), Vec3::Y),
                        camera: Camera {
                            is_active: true,
                            clear_color: ClearColorConfig::Custom(Color::srgba(0.2, 0.2, 0.2, 1.0)),
                            target: render_image.clone().into(),
                            ..default()
                        },
                        ..default()
                    })
                    .insert(GameWorldImage(render_image.clone()))
                    .insert(Flycam);

                route
                    .spawn((
                        UiTreeBundle::<MainUi>::from(UiTree::new("Hud")),
                        MovableByCamera,
                    ))
                    .with_children(|ui| {
                        let root = UiLink::<MainUi>::path("Hud");
                        // ui.spawn((root.clone(), UiLayout::window_full().pack::<Base>()));

                        // Spawn 3D camera view
                        ui.spawn((
                            root.add("3dGameWorldCamera"),
                            UiLayout::solid()
                                .size((1920.0, 1080.0))
                                .scaling(Scaling::Fill)
                                .pack::<Base>(),
                            UiImage2dBundle::from(render_image),
                            Pickable::IGNORE,
                        ));
                        let text = "Spinner";
                        const BUTTON_HEIGHT: f32 = 25.;
                        const BUTTON_SPACING: f32 = 10.;
                        let mut above = 0.0;

                        // Spawn spinner button
                        ui.spawn((
                            root.add(text),
                            UiDepthBias(50.0),
                            MaterialMesh2dBundle {
                                mesh: Mesh2dHandle(meshes.add(Rectangle {
                                    half_size: Vec2::splat(50.0),
                                })),
                                material: materials.add(Palette::Blue.to_color()),
                                ..default()
                            },
                            Element,
                            Dimension::default(),
                            UiLayout::window()
                                .pos((
                                    Rl(100.) - Ab(90.),
                                    Ab(BUTTON_SPACING + (BUTTON_HEIGHT + BUTTON_SPACING) * above),
                                ))
                                .size(Ab((80.0, BUTTON_HEIGHT)))
                                .pack::<Base>(),
                            crate::components::ui::button::Button { text: text.into() },
                        ));
                        above += 1.0;
                        let text = "Targets";
                        // Spawn spinner button
                        ui.spawn((
                            root.add("Targets"),
                            UiDepthBias(50.0),
                            MaterialMesh2dBundle {
                                mesh: Mesh2dHandle(meshes.add(Rectangle {
                                    half_size: Vec2::splat(50.0),
                                })),
                                material: materials.add(Palette::Blue.to_color()),
                                ..default()
                            },
                            OnUiClickCommands::new(|commands| {
                                info!("Spawning targets");
                                commands.spawn(crate::objects::Targets::default());
                            }),
                            Element,
                            Dimension::default(),
                            UiLayout::window()
                                .pos((
                                    Rl(100.) - Ab(90.),
                                    Ab(BUTTON_SPACING + (BUTTON_HEIGHT + BUTTON_SPACING) * above),
                                ))
                                .size(Ab((80.0, 25.)))
                                .pack::<Base>(),
                            crate::components::ui::button::Button { text: text.into() },
                        ));
                    });
            });
    }
}

fn flycam_controller() -> UnrealCameraController {
    UnrealCameraController {
        keyboard_mvmt_sensitivity: 100.0,
        ..default()
    }
}
