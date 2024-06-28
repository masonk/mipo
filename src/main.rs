#![feature(f16)]
#![feature(trait_alias)]
#![feature(iter_array_chunks)]
#![feature(array_chunks)]
use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};
use clap::Parser;
use std::path::PathBuf;
// // Preprocess an image for rtin meshing.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    terrain: PathBuf,
}

mod bevy_rtin;
mod camera;
mod geometry;
mod items;
mod physics;
mod player;
mod rtin;
mod world;

use bevy::log::LogPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use smooth_bevy_cameras::{controllers::unreal::UnrealCameraPlugin, LookTransformPlugin};

use camera::CameraPlugin;
use player::PlayerPlugin;
use world::WorldPlugin;

// cargo run assets/grand_canyon_small_heightmap.png
// cargo run assets/36_377_-112_445_11_8129_8129.png
fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: "info,wgpu_core=warn,wgpu_hal=warn,main=debug".into(),
                level: bevy::log::Level::DEBUG,
                ..default()
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default().disabled(),
            PlayerPlugin,
            CameraPlugin,
            items::spinner::SpinnerUiPlugin,
            LookTransformPlugin,
            UnrealCameraPlugin::default(),
            WireframePlugin,
            WorldPlugin {
                terrain_path: args.terrain,
            },
            // ThirdPersonCameraPlugin,
            WorldInspectorPlugin::new(),
        ))
        .insert_resource(WireframeConfig {
            // The global wireframe config enables drawing of wireframes on every mesh,
            // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
            // regardless of the global configuration.
            global: false,
            // Controls the default color of all wireframes. Used as the default color for global wireframes.
            // Can be changed per mesh using the `WireframeColor` component.
            ..default()
        })
        .run();
}
