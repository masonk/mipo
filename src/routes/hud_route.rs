use bevy::{
    math::vec3,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use bevy_lunex::prelude::*;
use smooth_bevy_cameras::controllers::unreal::{UnrealCameraBundle, UnrealCameraController};

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct HudRoute;
#[derive(Debug, Clone, Component)]
pub(crate) struct Flycam;

pub struct HudRoutePlugin;

impl Plugin for HudRoutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, build_route.before(UiSystems::Compute));
    }
}

fn build_route(
    mut commands: Commands,
    query: Query<Entity, Added<HudRoute>>,
    asset_server: Res<AssetServer>,
) {
    for route_entity in &query {
        // #======================#
        // #=== USER INTERFACE ===#

        info!("Spawning Hud route");
        // Spawn the route
        commands
            .entity(route_entity)
            .insert(SpatialBundle::default())
            .with_children(|route| {
                // Render 3D camera onto a texture
                let size = Extent3d {
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
                        let root = UiLink::<MainUi>::path("Root");
                        ui.spawn((root.clone(), UiLayout::window_full().pack::<Base>()));

                        // Spawn 3D camera view
                        ui.spawn((
                            root.add("Root/3dGameWorldCamera"),
                            UiLayout::solid()
                                .size((1920.0, 1080.0))
                                .scaling(Scaling::Fill)
                                .pack::<Base>(),
                            UiImage2dBundle::from(render_image),
                            Pickable::IGNORE,
                        ));

                        // Spawn panel
                        ui.spawn((
                            root.add("Return"),
                            UiLayout::window()
                                .pos(Rl((2.0, 4.0)))
                                .size(Rl((16.0, 8.0)))
                                .pack::<Base>(),
                            crate::components::ui::button::Button {
                                text: "MY BUTTON!".into(),
                            },
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
