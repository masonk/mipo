use crate::palette::Palette;
use bevy::{
    prelude::*,
    sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle, Sprite},
    window::WindowResized,
};

pub struct ManaPlugin;

impl Plugin for ManaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup_mana_bar);
        app.add_systems(FixedUpdate, regen);
        app.add_systems(Update, reposition_mana_bar);
    }
}
#[derive(Component, Default)]
pub struct Mana {
    pub current: u32,
    pub max: u32,
}

#[derive(Component, Default)]
pub struct ManaRegen {
    pub regen_mana_timer: Timer, // Time should tick every time more mana should be given
    pub regen_per_tick: u32,     // every time the timer ticks, give back this much mana.
}

#[derive(Component)]
pub struct ManaBarForeground;

#[derive(Component)]
struct ManaBarBackground;

const BAR_HALF_HEIGHT: f32 = 50.;
const BAR_HALF_WIDTH: f32 = 10.;
const BAR_MARGIN_X: f32 = 10.;
const BAR_MARGIN_Y: f32 = 10.;

fn reposition_mana_bar(
    mut resize_reader: EventReader<WindowResized>,
    mut foreground: Query<&mut Transform, With<ManaBarForeground>>,
    mut background: Query<&mut Transform, (With<ManaBarBackground>, Without<ManaBarForeground>)>,
) {
    if let Some(e) = resize_reader.read().last() {
        let bar_x = e.width / 2. - BAR_MARGIN_X;
        let bar_y = e.height / -2. + BAR_MARGIN_Y;
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
            let percent = mana.current as f32 / mana.max as f32;

            mana_bar_transform.scale.y = percent;
        }
    }
}

fn startup_mana_bar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        // MaterialMesh2dBundle {
        //     mesh: Mesh2dHandle(meshes.add(Rectangle::default()).into()),
        //     transform: Transform::from_translation(Vec3::new(0., 0., 3.)).with_scale(Vec3::new(
        //         BAR_HALF_WIDTH * 2.,
        //         BAR_HALF_HEIGHT * 2.,
        //         1.,
        //     )),
        //     material: materials.add(Palette::HudBackground.to_color()),
        //     ..default()
        // },
        ManaBarBackground,
        SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::BottomRight,
                color: Palette::HudBackground.to_color(),
                custom_size: Some(Vec2::new(BAR_HALF_WIDTH * 2., BAR_HALF_HEIGHT * 2.)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., 0., 3.)),
            ..default()
        },
    ));
    commands.spawn((
        // MaterialMesh2dBundle {
        //     mesh: Mesh2dHandle(meshes.add(Rectangle::default()).into()),
        //     transform: Transform::from_translation(Vec3::new(0., 0., 4.)).with_scale(Vec3::new(
        //         BAR_HALF_WIDTH * 2.,
        //         BAR_HALF_HEIGHT * 2.,
        //         1.,
        //     )),
        //     material: materials.add(Palette::Blue.to_color()),
        //     ..default()
        // },
        SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::BottomRight,
                color: Palette::Blue.to_color(),
                custom_size: Some(Vec2::new(BAR_HALF_WIDTH * 2., BAR_HALF_HEIGHT * 2.)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., 0., 4.)),
            ..default()
        },
        ManaBarForeground,
    ));
}
