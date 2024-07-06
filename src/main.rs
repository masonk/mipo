#![feature(f16)]
#![feature(trait_alias)]
#![feature(iter_array_chunks)]
#![feature(array_chunks)]
use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    window::{Cursor, CursorGrabMode, WindowResolution},
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
mod components;
mod geometry;
mod items;
mod objects;
mod palette;
mod physics;
mod player;
mod routes;
mod rtin;
mod world;

use bevy::log::LogPlugin;
// use bevy_inspector_egui;
use bevy_lunex;
use bevy_rapier3d::prelude::*;
use bevy_stl;
use smooth_bevy_cameras;

// cargo run  --target wasm32-unknown-unknown  assets/grand_canyon_small_heightmap.png
// cargo run assets/grand_canyon_small_heightmap.png
// cargo run assets/36_377_-112_445_11_8129_8129.png
fn main() {
    // let args = Args::parse();

    App::new()
        // Enable ambiguity warnings for the Update schedule
        // .edit_schedule(Update, |schedule| {
        //     schedule.set_build_settings(ScheduleBuildSettings {
        //         ambiguity_detection: LogLevel::Warn,
        //         ..default()
        //     });
        // })
        .add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    filter: "info,wgpu_core=warn,wgpu_hal=warn,main=debug".into(),
                    level: bevy::log::Level::DEBUG,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        position: WindowPosition::At((0, 0).into()),
                        cursor: Cursor {
                            grab_mode: CursorGrabMode::Confined,
                            ..default()
                        },
                        // mode: WindowMode::BorderlessFullscreen,
                        resolution: WindowResolution::new(1920. * 1.8, 1080.0 * 1.8)
                            .with_scale_factor_override(1.0),
                        // resolution: bevy::window::WindowResolution::new(1920., 1080.),
                        // fill the entire browser window
                        // TODO: re-enable in Bevy 0.14
                        // fit_canvas_to_parent: true,
                        // don't hijack keyboard shortcuts like F5, F6, F12, Ctrl+R etc.
                        // fit_canvas_to_parent: true,
                        // prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
            bevy_rapier3d::plugin::RapierPhysicsPlugin::<NoUserData>::default(),
            bevy_rapier3d::render::RapierDebugRenderPlugin::default().disabled(),
            player::PlayerPlugin,
            smooth_bevy_cameras::LookTransformPlugin,
            smooth_bevy_cameras::controllers::unreal::UnrealCameraPlugin::default(),
            WireframePlugin,
            world::WorldPlugin {
                terrain_path: "assets/grand_canyon_small_heightmap.png".into(),
            },
            // ThirdPersonCameraPlugin,
            // bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
            bevy_stl::StlPlugin,
            components::ComponentPlugin,
        ))
        // .add_system_to_stage(
        //     CoreStage::PostUpdate,
        //     Assets::<Image>::asset_event_system.before(CameraUpdateSystem),
        // )
        .add_plugins(camera::CameraPlugin)
        .add_systems(Startup, startup)
        .add_plugins((
            objects::TargetsPlugin,
            items::ItemsPlugin,
            routes::RoutesPlugin,
            bevy_lunex::UiPlugin, // diegetic ui system
        ))
        .insert_resource(ClearColor(Color::srgb(0.53, 0.53, 0.53)))
        .insert_resource(WireframeConfig {
            global: false,
            ..default()
        })
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(routes::hud_route::HudRoute);
}
