use crate::prelude::*;
use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Debug, Copy, Clone, Default)]
enum EnemyBehavior {
    #[default]
    InterceptingPlayer,
}

#[derive(Component, Debug, Copy, Clone, Default)]
pub struct Enemy {
    visual_aggro_radius: f32,
    behavior: EnemyBehavior,
}

#[derive(Bundle, Default)]
pub struct EnemyBundle {
    enemy: Enemy,
    hitpoints: crate::hitpoints::Hp,
}
