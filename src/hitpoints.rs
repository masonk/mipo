use bevy::prelude::*;

pub struct HpPlugin;

impl Plugin for HpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, regen);
    }
}
#[derive(Component, Default)]
pub struct Hp {
    pub current: u32,
    pub max: u32,
}

#[derive(Component, Default)]
pub struct HpRegen {
    pub tick_timer: Timer,
    pub regen_per_tick: u32,
}

fn regen(time: Res<Time>, mut hp_query: Query<(&mut Hp, &mut HpRegen)>) {
    for (mut hp, mut regen) in &mut hp_query {
        // give the player some mana back
        regen.tick_timer.tick(time.delta());
        if regen.tick_timer.finished() {
            hp.current += regen.regen_per_tick;
            hp.current = hp.current.clamp(0, hp.max);
        }
    }
}
