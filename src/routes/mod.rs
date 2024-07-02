pub mod hud_route;
use bevy::prelude::*;

pub struct RoutesPlugin;
impl Plugin for RoutesPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugins(spinner_config::SpinnerConfigRoutePlugin);
        app.add_plugins(hud_route::HudRoutePlugin);
    }
}
/// When this component is added, a UI system is built
#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct Button {
    pub text: String,
}
