use bevy::{
    input::{mouse::MouseMotion, InputSystem},
    log::prelude::*,
    prelude::*,
};

#[derive(Component)]
pub struct SpinnerUi;

pub struct SpinnerUiPlugin;

impl Plugin for SpinnerUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup));
    }
}

pub fn setup() {}
