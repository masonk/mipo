use crate::*;
use bevy::sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle};
use bevy_lunex::prelude::*;
use palette::Palette;

// #=========================#
// #=== EXPOSED COMPONENT ===#

/// When this component is added, a UI system is built
#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct Button {
    pub text: String,
}

/// Marker struct for the sandboxed UI
#[derive(Component, Debug, Default, Clone, PartialEq)]
struct ButtonUi;

/// System that builds the component UI
fn build_component(
    mut commands: Commands,
    query: Query<(Entity, &Button), Added<Button>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, button_source) in &query {
        // This will create a private sandboxed UiTree within the entity just for the button
        commands
            .entity(entity)
            .insert(UiTreeBundle::<ButtonUi>::from(UiTree::new("Button")))
            .with_children(|ui| {
                info!("Spawning a button with text: {}", button_source.text);
                let sandbox_root = UiLink::<ButtonUi>::path("Control");

                // Spawn button text
                let text = ui
                    .spawn((
                        // Link this widget
                        sandbox_root.add("Text"),
                        // Here we can define where we want to position our text within the parent node,
                        // don't worry about size, that is picked up and overwritten automaticaly by Lunex to match text size.
                        UiLayout::window()
                            .pos(Rl((0., 0.)))
                            .anchor(Anchor::Center)
                            .pack::<Base>(),
                        // Add text
                        UiText2dBundle {
                            text: Text::from_section(
                                &button_source.text,
                                TextStyle {
                                    // font: assets.font_medium.clone(),
                                    font_size: 60.0, // Currently hardcoded as Relative height (Rh) - so 60% of the node height
                                    ..default()
                                },
                            ),
                            ..default()
                        },
                        // Make it non-obsructable for hit checking (mouse detection)
                        Pickable::IGNORE,
                        // This is required to control our hover animation
                        UiAnimator::<Hover>::new().receiver(true),
                        // This will set the color to red
                        // UiColor::<Base>::new(Color::BEVYPUNK_RED),
                        UiColor::<Base>::new(Palette::Red.into()),
                        // // This will set hover color to yellow
                        UiColor::<Hover>::new(Palette::Yellow.into()),
                    ))
                    .id();

                // Spawn button hover-zone
                ui.spawn((
                    // Link this widget
                    sandbox_root,
                    // Add layout
                    UiLayout::window_full().pack::<Base>(),
                    // Make this spacial & clickable entity
                    UiZoneBundle::default(),
                    // This is required to control our hover animation
                    UiAnimator::<Hover>::new()
                        .forward_speed(5.0)
                        .backward_speed(1.0),
                    // This will pipe this hover data to the specified entities
                    UiAnimatorPipe::<Hover>::new(vec![text]),
                    // This will change cursor icon on mouse hover
                    OnHoverSetCursor::new(CursorIcon::Pointer),
                    // If we click on this hover zone, it will emmit UiClick event from parent entity
                    UiClickEmitter::new(entity),
                ));
            });
    }
}

// #========================#
// #=== COMPONENT PLUGIN ===#

/// Plugin adding all our logic
pub struct ButtonPlugin;
impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add Lunex plugins for our sandboxed UI
            .add_plugins(UiGenericPlugin::<ButtonUi>::new())
            // Add general systems
            .add_systems(Update, build_component.before(UiSystems::Compute));
    }
}
