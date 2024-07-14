use bevy::sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle};
use bevy::{
    asset::AssetEvent,
    math::vec3,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    window::{PrimaryWindow, WindowResized},
};

use bevy_lunex::prelude::*;
use smooth_bevy_cameras::controllers::unreal::{UnrealCameraBundle, UnrealCameraController};

use crate::camera::{FirstPersonCam, Flycam};
use crate::palette::Palette;
use crate::GameState;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct HudRoute;

pub struct HudRoutePlugin;

#[derive(Resource)]
pub struct GameWorldImage(pub Handle<Image>);

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct GameWorldRoute;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct CamerasSet;

impl Plugin for HudRoutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, build_route.before(UiSystems::Compute));
        app.add_systems(OnEnter(GameState::StartingUp), enter_starting_up);
        app.add_systems(
            OnEnter(GameState::DevMode),
            enter_dev_mode.in_set(CamerasSet),
        );
        app.add_systems(
            OnEnter(GameState::Prespawn),
            enter_spawning.in_set(CamerasSet),
        );
        app.add_systems(Update, update_gameworld_viewport);
    }
}

fn update_gameworld_viewport(
    mut commands: Commands,
    mut asset_events: EventWriter<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
    mut resize_reader: EventReader<WindowResized>,
    window: Query<&Window, With<PrimaryWindow>>,
    game_world: Option<ResMut<GameWorldImage>>,
    game_world_route: Query<(Entity, Option<&UiLink>), With<GameWorldRoute>>,
) {
    let game_world = match game_world {
        Some(v) => v,
        None => return,
    };

    match game_world_route.get_single() {
        Ok((game_world_entity, ui_link)) => {
            if let Some(e) = resize_reader.read().last() {
                debug!("Window resized. New extent: {}, {}", e.width, e.height);

                asset_events.send(AssetEvent::Modified {
                    id: game_world.0.id(),
                });

                if let Some(image) = images.get_mut(game_world.0.id()) {
                    let size = Extent3d {
                        width: e.width as u32,
                        height: e.height as u32,
                        ..default()
                    };
                    image.resize(size);
                }

                commands
                    .entity(game_world_entity)
                    .insert(game_world_solid_bundle(
                        e.width,
                        e.height,
                        game_world.0.clone(),
                    ));
            } else if ui_link.is_none() {
                debug!("No UiLink for GameWorldRoute: inserting bundle");
                if let Ok(window) = window.get_single() {
                    commands
                        .entity(game_world_entity)
                        .insert(game_world_solid_bundle(
                            window.width(),
                            window.height(),
                            game_world.0.clone(),
                        ));
                }
            }
        }
        Err(e) => return warn!("Couldn't find GameWorldRoute: {e}"),
    }
}

fn game_world_solid_bundle(w: f32, h: f32, image: Handle<Image>) -> impl Bundle {
    (
        UiLink::<MainUi>::path("Hud/GameWorld"),
        UiLayout::solid()
            .size((w, h))
            .scaling(Scaling::VerFill)
            .pack::<Base>(),
        UiImage2dBundle::from(image),
        Pickable::IGNORE,
    )
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

fn enter_starting_up(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window: &Window = windows.single();
    let render_image =
        asset_server.add(camera_image(window.width() as u32, window.height() as u32));
    let new_game_world = GameWorldImage(render_image.clone());
    commands.insert_resource(new_game_world);
}

fn enter_dev_mode(
    mut commands: Commands,
    hud_entity: Query<Entity, With<HudRoute>>,
    game_world: Res<GameWorldImage>,
    cam: Query<&Camera, With<Flycam>>,
) {
    if let Ok(_) = cam.get_single() {
        return;
    }

    for route_entity in &hud_entity {
        commands
            .entity(route_entity)
            .insert(SpatialBundle::default())
            .with_children(|route| {
                route
                    .spawn(Camera3dBundle {
                        projection: PerspectiveProjection {
                            // We must specify the FOV in radians.
                            // Rust can convert degrees to radians for us.
                            fov: 50.0_f32.to_radians(),
                            ..default()
                        }
                        .into(),
                        camera: Camera {
                            is_active: true,
                            order: 111,
                            clear_color: ClearColorConfig::Custom(Color::srgba(0.2, 0.2, 0.2, 1.0)),
                            target: game_world.0.clone().into(),
                            ..default()
                        },
                        ..default()
                    })
                    .insert((
                        // StateScoped(GameState::DevMode),
                        Flycam,
                        UnrealCameraBundle::new(
                            flycam_controller(),
                            vec3(-154.44, 204.027, -111.268),
                            vec3(150., 20.0, 150.0),
                            Vec3::Y,
                        ),
                    ));
            });
    }
}

fn enter_spawning(
    commands: Commands,
    hud_entity: Query<Entity, With<HudRoute>>,
    game_world: Res<GameWorldImage>,
    cam: Query<&Camera, With<Flycam>>,
) {
    info!("PRESPAWNING: creating flycam");
    enter_dev_mode(commands, hud_entity, game_world, cam);
}

fn build_route(
    mut commands: Commands,
    query: Query<Entity, Added<HudRoute>>,
    mut meshes: ResMut<Assets<Mesh>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = match windows.get_single() {
        Ok(w) => w,
        Err(e) => return warn!("Couldnt' get primary window: {e}"),
    };

    for route_entity in &query {
        info!("Spawning Hud Route");
        // let window = window.single();
        // Spawn the route
        commands
            .entity(route_entity)
            .insert(SpatialBundle::default())
            .with_children(|route| {
                Vec2::new(window.width() / 2.0 - 500., window.height() / -2.0 + 500.);

                route
                    .spawn((
                        UiTreeBundle::<MainUi>::from(UiTree::new("Hud")),
                        MovableByCamera,
                    ))
                    .with_children(|ui| {
                        let root = UiLink::<MainUi>::path("Hud");
                        info!("Spawned GameWorldRoute as a child of UiTree");

                        ui.spawn(GameWorldRoute);
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
                                    half_size: Vec2::new(50., 25.),
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
                                .size(Ab((100., 50.)))
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
#[derive(Component, Debug, Default, Clone, PartialEq)]
struct ManaRemaining;
