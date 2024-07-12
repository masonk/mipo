use std::time::Duration;

use crate::palette::Palette;
use bevy::prelude::*;
use bevy::{
    prelude::*,
    sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle},
    window::WindowResized,
};

pub struct ManaPlugin;

impl Plugin for ManaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, regen);
        app.add_systems(Update, (setup_mana_bar, reposition_mana_bar));
    }
}
#[derive(Component, Default)]
pub struct Mana {
    pub current: u32,
    pub max: u32,
}

#[derive(Component, Default)]
pub struct ManaRegen {
    pub regen_mana_timer: Timer, // every time the timer ticks, give back this much mana.
    pub regen_per_tick: u32,
}

#[derive(Component)]
pub struct ManaBarForeground;

#[derive(Component)]
struct ManaBarBackground;

const BAR_HALF_HEIGHT: f32 = 150.;
const BAR_HALF_WIDTH: f32 = 50.;
const BAR_MARGIN_X: f32 = 10.;
const BAR_MARGIN_Y: f32 = 10.;

fn reposition_mana_bar(
    mut resize_reader: EventReader<WindowResized>,
    mut foreground: Query<&mut Transform, With<ManaBarForeground>>,
    mut background: Query<&mut Transform, (With<ManaBarBackground>, Without<ManaBarForeground>)>,
) {
    if let Some(e) = resize_reader.read().last() {
        let bar_x = e.width / 2. - BAR_HALF_WIDTH - BAR_MARGIN_X;
        let bar_y = e.height / -2. + BAR_HALF_HEIGHT + BAR_MARGIN_Y;
        if let Ok(mut background) = background.get_single_mut() {
            background.translation.x = bar_x;
            background.translation.y = bar_y;
        };
        if let Ok(mut foreground) = foreground.get_single_mut() {
            foreground.translation.x = bar_x;
            foreground.translation.y = bar_y;
        }
    }
}

fn regen(
    time: Res<Time>,
    mut mana_query: Query<(&mut Mana, &mut ManaRegen)>,
    mut mana_bar: Query<&mut Transform, With<ManaBarForeground>>,
) {
    for (mut mana, mut regen) in &mut mana_query {
        // give the player some mana back
        regen.regen_mana_timer.tick(time.delta());
        if regen.regen_mana_timer.finished() {
            mana.current += regen.regen_per_tick;
            mana.current = mana.current.clamp(0, mana.max);
        }
        if let Ok(mut mana_bar_transform) = mana_bar.get_single_mut() {
            mana_bar_transform.scale.y =
                BAR_HALF_HEIGHT * 2. * (mana.current as f32 / mana.max as f32);
        }
    }
}

fn setup_mana_bar(
    mut commands: Commands,
    camera_query: Query<Entity, Added<Camera2d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if let Ok(camera_entity) = camera_query.get_single() {
        commands.entity(camera_entity).with_children(|camera| {
            camera.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::default()).into()),
                    transform: Transform::from_translation(Vec3::new(0., 0., 1.))
                        .with_scale(Vec3::new(BAR_HALF_WIDTH * 2., BAR_HALF_HEIGHT * 2., 1.)),
                    material: materials.add(Palette::HudBackground.to_color()),
                    ..default()
                },
                ManaBarBackground,
            ));
            camera.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::default()).into()),
                    transform: Transform::from_translation(Vec3::new(0., 0., 2.))
                        .with_scale(Vec3::new(BAR_HALF_WIDTH * 2., BAR_HALF_HEIGHT * 2., 1.)),
                    material: materials.add(Palette::Blue.to_color()),
                    ..default()
                },
                ManaBarForeground,
            ));
        });
    }
}
