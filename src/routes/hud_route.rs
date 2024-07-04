use bevy::sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle};
use bevy::{
    math::vec3,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};

use bevy_lunex::prelude::*;
use smooth_bevy_cameras::controllers::unreal::{UnrealCameraBundle, UnrealCameraController};

use crate::camera::Flycam;
use crate::palette::Palette;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct HudRoute;

pub struct HudRoutePlugin;

impl Plugin for HudRoutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, build_route.before(UiSystems::Compute));
    }
}

fn build_route(
    mut commands: Commands,
    query: Query<Entity, Added<HudRoute>>,
    // window: Query<&Window>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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
                // Render 3D camera onto a texture
                let size = Extent3d {
                    // height: window.resolution.height() as u32,
                    // width: window.resolution.width() as u32,
                    width: 1920,
                    height: 1080,
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
                let render_image = asset_server.add(image);

                info!("Spawning 3d camera");
                route
                    .spawn(Camera3dBundle {
                        camera: Camera {
                            is_active: true,
                            clear_color: ClearColorConfig::Custom(Color::srgba(0.2, 0.4, 0.2, 1.0)),
                            target: render_image.clone().into(),
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
