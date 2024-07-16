use crate::{player_hud::ManaBar, prelude::*};
use bevy::{
    prelude::*,
    sprite::{Anchor, Sprite},
    window::WindowResized,
};

pub struct ManaPlugin;

impl Plugin for ManaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, regen);
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

fn regen(time: Res<Time>, mut mana_query: Query<(&mut Mana, &mut ManaRegen)>) {
    for (mut mana, mut regen) in &mut mana_query {
        // give the player some mana back
        regen.regen_mana_timer.tick(time.delta());
        if regen.regen_mana_timer.finished() {
            mana.current += regen.regen_per_tick;
            mana.current = mana.current.clamp(0, mana.max);
        }
    }
}
