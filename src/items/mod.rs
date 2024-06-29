use bevy::prelude::*;
pub mod spinner;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(spinner::SpinnerUiPlugin);
    }
}
