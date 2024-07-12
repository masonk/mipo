use std::time::Duration;

use bevy::prelude::*;

use bevy_rapier3d::prelude::*;
pub struct FireballPlugin;
use crate::{camera::FirstPersonCam, mana::Mana};
impl Plugin for FireballPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (throw_fireball, clean_fireball));
    }
}

#[derive(Component)]
pub struct Fireball {
    lifetime: Timer,
}

#[derive(Component, Debug, Clone)]
pub struct FireballAbility {
    pub projectile_lifetime: Timer,
    pub projectile_speed: f32,
    pub projectile_radius: f32,
    pub mana_cost: u32,
    pub cooldown_timer: Timer, // should be a repeating timer.
    pub damping: f32,
    pub gravity: f32,
}

impl Default for FireballAbility {
    fn default() -> Self {
        FireballAbility {
            projectile_lifetime: Timer::new(Duration::from_millis(5000), TimerMode::Once),
            projectile_speed: 20.,
            projectile_radius: 1.,
            mana_cost: 5,
            cooldown_timer: Timer::new(Duration::from_millis(3000), TimerMode::Repeating),
            damping: 2.,
            gravity: 1.,
        }
    }
}

pub fn clean_fireball(
    mut commands: Commands,
    mut fireballs: Query<(Entity, &mut Fireball)>,
    time: Res<Time>,
) {
    for (entity, mut fireball) in &mut fireballs {
        fireball.lifetime.tick(time.delta());
        if fireball.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn throw_fireball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut thrower_query: Query<(&mut FireballAbility, &mut Mana), With<FireballAbility>>,
    camera_query: Query<(&GlobalTransform), With<FirstPersonCam>>,
    time: Res<Time>,
) {
    let (mut ability, mut mana) = match thrower_query.get_single_mut() {
        Ok(p) => p,
        Err(e) => return,
    };
    let transform: GlobalTransform = match camera_query.get_single() {
        Ok(v) => v.clone(),
        Err(e) => {
            return warn!("Couldn't get FirstPersonCam, don't know how to aim the fireball: {e}")
        }
    };

    // Repeating timers are only ready in the first tick after they finish, then they become unready again.
    // So, only tick if still in cooldown.
    if !ability.cooldown_timer.finished() {
        ability.cooldown_timer.tick(time.delta());
    }

    if !ability.cooldown_timer.finished() {
        return;
    }
    if !mouse.pressed(MouseButton::Right) {
        return;
    }
    if mana.current < ability.mana_cost {
        info!("Can't spawn a fireball, out of mana.");
        return;
    }
    // Put the fireball back on cooldown.
    ability.cooldown_timer.reset();
    mana.current -= ability.mana_cost;

    info!("Throwing a fireball. Current mana: {}", mana.current);
    let local_transform: Transform = transform.into();
    let forward = transform.forward();
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(
                    Sphere::new(ability.projectile_radius)
                        .mesh()
                        .ico(6)
                        .unwrap(),
                ),
                transform: local_transform,
                material: materials.add(StandardMaterial {
                    diffuse_transmission: 0.0,
                    specular_transmission: 1.0,
                    thickness: 0.7,
                    reflectance: 1.0,
                    perceptual_roughness: 0.24,
                    ..default()
                }),

                ..default()
            },
            Fireball {
                lifetime: ability.projectile_lifetime.clone(),
            },
        ))
        .insert((
            RigidBody::Dynamic,
            Collider::ball(ability.projectile_radius),
            GravityScale(ability.gravity),
            // prevents "tunneling"
            Ccd::enabled(),
            ColliderMassProperties::Density(5.),
            Damping {
                linear_damping: ability.damping,
                ..default()
            },
        ))
        .insert(Velocity {
            linvel: forward * ability.projectile_speed,
            ..default()
        })
        .with_children(|builder| {
            // particle effects here...
        });
}
