use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_lunex::prelude::*;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct SpinnerConfigRoute;
use super::*;

// #====================#
// #=== ROUTE PLUGIN ===#

/// Plugin adding all our logic
pub struct SpinnerConfigRoutePlugin;
impl Plugin for SpinnerConfigRoutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, build_route.before(UiSystems::Compute));
        // .add_systems(Update, main_menu_button_clicked_system.run_if(on_event::<UiClickEvent>()));
    }
}

fn build_route(
    mut commands: Commands,
    // assets: Res<AssetCache>,
    main: Query<Entity, With<MainUi>>,
    query: Query<Entity, Added<SpinnerConfigRoute>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for route_entity in &query {
        // Spawn the route
        commands
            .entity(route_entity)
            .insert(SpatialBundle::default())
            .with_children(|route| {
                // Spawn the master ui tree
                route
                    .spawn((
                        UiTreeBundle::<MainUi>::from(UiTree::new("SpinnerConfig")),
                        MovableByCamera,
                    ))
                    .with_children(|ui| {
                        let root = UiLink::<MainUi>::path("SpinnerConfig"); // Here we can define the name of the node
                        ui.spawn((
                            root.clone(),                           // Here we add the link
                            UiLayout::window_full().pack::<Base>(), // This is where we define layout
                        ));

                        ui.spawn((
                            root.add("Spinner"),
                            UiLayout::solid().size((1920.0, 1080.0)).pack::<Base>(),
                            UiDepthBias(1.0),
                            Element::default(),
                            Dimension::default(),
                        ));
                    });
            });
    }
}

// commands.spawn((
//     // This makes the UI entity able to receive camera data
//     MovableByCamera,
//     // This is our UI system
//     UiTreeBundle::<MainUi>::from(UiTree::new("Hello UI!")),
// ));
