pub mod ui;
use bevy::prelude::*;

/// Plugin adding all our component logic
pub struct ComponentPlugin;
impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ui::button::ButtonPlugin);
    }
}
