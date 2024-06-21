#![feature(f16)]
#![feature(trait_alias)]
use bevy::prelude::*;

mod bevy_rtin;
mod camera;
mod geometry;
mod player;
mod rtin;
mod world;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_log;
use bevy_third_person_camera::ThirdPersonCameraPlugin;
use env_logger;

use camera::CameraPlugin;
use player::PlayerPlugin;
use world::WorldPlugin;
fn main() {
    env_logger::init();
    App::new()
        .add_plugins((
            DefaultPlugins.build().disable::<bevy_log::LogPlugin>(),
            PlayerPlugin,
            CameraPlugin,
            WorldPlugin,
            ThirdPersonCameraPlugin,
            WorldInspectorPlugin::new(),
        ))
        .run();
}
