use crate::{hitpoints::Hp, mana::Mana, prelude::*};
use bevy::prelude::*;

pub const RIGHT_HAND_SIDE_WIDTH: f32 = 200.;
pub const BAR_HEIGHT: f32 = 100.;
pub const BAR_WIDTH: f32 = 20.;
pub const BAR_MARGIN_X: f32 = 10.;
pub const BAR_MARGIN_Y: f32 = 10.;

pub struct PlayerHudPlugin;

#[derive(Component)]
pub struct HpBar;

#[derive(Component)]
pub struct ManaBar;

impl Plugin for PlayerHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), insert_bars);
        app.add_systems(OnExit(GameState::InGame), remove_bars);
        app.add_systems(Update, update_bars.run_if(in_state(GameState::InGame)));
    }
}

fn insert_bars(mut commands: Commands) {
    info!("Inserting root node");
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                NodeBundle { ..default() },
                Name::new("player_hud_left_column"),
            ));
            let mut color = Palette::HudBackground.to_color();
            color.set_alpha(0.1);
            p.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(RIGHT_HAND_SIDE_WIDTH),
                        height: Val::Percent(100.),
                        justify_content: JustifyContent::SpaceBetween,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: BackgroundColor(color),
                    ..default()
                },
                Name::new("player_hud_right_column"),
            ))
            .with_children(|p| {
                p.spawn((NodeBundle { ..default() }, Name::new("top_right")));
                p.spawn((
                    NodeBundle {
                        style: Style {
                            height: Val::Px(BAR_HEIGHT + (BAR_MARGIN_Y * 2.0)),
                            justify_content: JustifyContent::SpaceAround,
                            align_content: AlignContent::FlexStart,
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        background_color: BackgroundColor(color),
                        ..default()
                    },
                    Name::new("bottom_right"),
                ))
                .with_children(|p| {
                    p.spawn((
                        NodeBundle {
                            style: Style {
                                margin: UiRect {
                                    top: Val::Px(BAR_MARGIN_Y),
                                    bottom: Val::Px(BAR_MARGIN_Y),
                                    left: Val::Px(BAR_MARGIN_X),
                                    right: Val::Px(BAR_MARGIN_X),
                                },
                                align_self: AlignSelf::FlexEnd,
                                width: Val::Px(BAR_WIDTH),
                                height: Val::Px(BAR_HEIGHT),
                                ..default()
                            },
                            background_color: Palette::Red.into(),
                            ..default()
                        },
                        HpBar,
                        Name::new("player_hp_bar"),
                    ));

                    p.spawn((
                        NodeBundle {
                            style: Style {
                                margin: UiRect {
                                    top: Val::Px(BAR_MARGIN_Y),
                                    bottom: Val::Px(BAR_MARGIN_Y),
                                    left: Val::Px(BAR_MARGIN_X),
                                    right: Val::Px(BAR_MARGIN_X),
                                },
                                align_self: AlignSelf::FlexEnd,
                                width: Val::Px(BAR_WIDTH),
                                height: Val::Px(BAR_HEIGHT),
                                ..default()
                            },
                            background_color: Palette::Blue.into(),
                            ..default()
                        },
                        ManaBar,
                        Name::new("player_mana_bar"),
                    ));
                });
            });
        });
}

fn remove_bars(
    mut commands: Commands,
    mut hp: Query<Entity, With<HpBar>>,
    mut mana: Query<Entity, With<ManaBar>>,
) {
    if let Ok(hp) = hp.get_single() {
        commands.entity(hp).despawn_recursive();
    }
    if let Ok(mana) = mana.get_single() {
        commands.entity(mana).despawn_recursive();
    }
}
fn update_bars(
    player_mana: Query<&Mana, With<Player>>,
    player_hp: Query<&Hp, With<Player>>,
    mut hp_bar: Query<&mut Style, (With<HpBar>, Without<ManaBar>)>,
    mut mana_bar: Query<&mut Style, With<ManaBar>>,
) {
    if let Ok(mut mana_bar) = mana_bar.get_single_mut() {
        if let Ok(mana) = player_mana.get_single() {
            let percent = mana.current as f32 / mana.max as f32;
            mana_bar.height = Val::Px(BAR_HEIGHT * percent);
        }
    }
    if let Ok(mut hp_bar) = hp_bar.get_single_mut() {
        if let Ok(hp) = player_hp.get_single() {
            let percent = hp.current as f32 / hp.max as f32;
            hp_bar.height = Val::Px(BAR_HEIGHT * percent);
        }
    }
}
