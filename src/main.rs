#![feature(f16)]
#![feature(trait_alias)]
#![feature(iter_array_chunks)]
use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use bevy_rapier3d::prelude::*;
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
mod physics;
mod player;
mod rtin;
mod world;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_log;
use bevy_third_person_camera::ThirdPersonCameraPlugin;
use env_logger;
use smooth_bevy_cameras::{controllers::unreal::UnrealCameraPlugin, LookTransformPlugin};

use camera::CameraPlugin;
use player::PlayerPlugin;
use world::WorldPlugin;

// cargo run assets/grand_canyon_small_heightmap.png
// cargo run assets/36_377_-112_445_11_8129_8129.png
fn main() {
    env_logger::init();
    let args = Args::parse();

    App::new()
        .add_plugins((
            DefaultPlugins.build().disable::<bevy_log::LogPlugin>(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            PlayerPlugin,
            CameraPlugin,
            LookTransformPlugin,
            UnrealCameraPlugin::default(),
            WireframePlugin,
            WorldPlugin {
                terrain_path: args.terrain,
            },
            // ThirdPersonCameraPlugin,
            WorldInspectorPlugin::new(),
        ))
        .run();
}
