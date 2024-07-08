use bevy::prelude::*;
pub mod spinner;

pub mod fireball;
pub use fireball::*;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(spinner::SpinnerUiPlugin);
        app.add_plugins(FireballPlugin);
    }
}
