//! HUD — bottom panel with HP bars, mana circles, crit counters.
//! Wave 3: static display from initial battle state.
#![allow(dead_code)]

use bevy::prelude::*;

use crate::shared::{Element, EnemyId, UnitId};

use super::plugin::{BattleRes, GameDataRes};

/// Marker for the HUD root node.
#[derive(Component)]
pub struct HudRoot;

/// Marker for HP bar fill (inner bar).
#[derive(Component)]
pub struct HpBarFill {
    pub side: HudSide,
    pub index: usize,
}

/// Marker for crit counter text.
#[derive(Component)]
pub struct CritCounterText {
    pub index: usize,
}

/// Marker for mana circle indicators.
#[derive(Component)]
pub struct ManaCircleRow;

#[derive(Clone, Copy)]
pub enum HudSide {
    Player,
    Enemy,
}

struct UnitCardSpec<'a> {
    name: &'a str,
    element: Element,
    hp_pct: f32,
    current_hp: u16,
    max_hp: u16,
    crit_counter: u8,
    index: usize,
    side: HudSide,
}

/// Element color helper (same as battle_scene).
fn element_color(element: Element) -> Color {
    match element {
        Element::Venus => Color::srgb(0.545, 0.451, 0.333),
        Element::Mars => Color::srgb(0.8, 0.267, 0.267),
        Element::Mercury => Color::srgb(0.267, 0.533, 0.8),
        Element::Jupiter => Color::srgb(0.8, 0.8, 0.267),
    }
}

fn lookup_element(id: &str, game_data: &GameDataRes) -> Element {
    if let Some(u) = game_data.0.units.get(&UnitId(id.to_string())) {
        return u.element;
    }
    if let Some(e) = game_data.0.enemies.get(&EnemyId(id.to_string())) {
        return e.element;
    }
    Element::Venus
}

fn lookup_name(id: &str, game_data: &GameDataRes) -> String {
    if let Some(u) = game_data.0.units.get(&UnitId(id.to_string())) {
        return u.name.clone();
    }
    if let Some(e) = game_data.0.enemies.get(&EnemyId(id.to_string())) {
        return e.name.clone();
    }
    id.to_string()
}

/// Spawn the HUD overlay.
pub fn setup_hud(
    mut commands: Commands,
    battle: Res<BattleRes>,
    game_data: Res<GameDataRes>,
) {
    let battle = &battle.0;

    // Full-screen column layout: main area + bottom HUD
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            HudRoot,
        ))
        .with_children(|root| {
            // Bottom HUD panel
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(120.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceEvenly,
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.10, 0.90)),
            ))
            .with_children(|panel| {
                // Player unit cards
                for (i, unit) in battle.player_units.iter().enumerate() {
                    let element = lookup_element(&unit.unit.id, &game_data);
                    let name = lookup_name(&unit.unit.id, &game_data);
                    let hp_pct = unit.unit.current_hp as f32 / unit.unit.stats.hp as f32;

                    spawn_unit_card(
                        panel,
                        UnitCardSpec {
                            name: &name,
                            element,
                            hp_pct,
                            current_hp: unit.unit.current_hp,
                            max_hp: unit.unit.stats.hp,
                            crit_counter: unit.unit.crit_counter,
                            index: i,
                            side: HudSide::Player,
                        },
                    );
                }

                // Mana pool display (center)
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::horizontal(Val::Px(16.0)),
                        ..default()
                    })
                    .with_children(|mana_col| {
                        // "MANA" label
                        mana_col.spawn((
                            Text::new("MANA"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.6)),
                        ));

                        // Mana circles row
                        mana_col
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(4.0),
                                    margin: UiRect::top(Val::Px(4.0)),
                                    ..default()
                                },
                                ManaCircleRow,
                            ))
                            .with_children(|row| {
                                let current = battle.mana_pool.current_mana;
                                let max = battle.mana_pool.max_mana;
                                for j in 0..max {
                                    let filled = j < current;
                                    let color = if filled {
                                        Color::srgb(0.267, 0.533, 0.8) // #4488cc
                                    } else {
                                        Color::srgb(0.2, 0.2, 0.25)
                                    };
                                    row.spawn((
                                        Node {
                                            width: Val::Px(12.0),
                                            height: Val::Px(12.0),
                                            ..default()
                                        },
                                        BackgroundColor(color),
                                        BorderRadius::all(Val::Px(6.0)),
                                    ));
                                }
                            });

                        // Numeric display
                        let mana_str = format!(
                            "{}/{}",
                            battle.mana_pool.current_mana, battle.mana_pool.max_mana
                        );
                        mana_col.spawn((
                            Text::new(mana_str),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.267, 0.533, 0.8)),
                        ));
                    });
            });
        });
}

/// Spawn a single unit card in the HUD panel.
fn spawn_unit_card(parent: &mut ChildBuilder, spec: UnitCardSpec<'_>) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            min_width: Val::Px(100.0),
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|card| {
            // Unit name with element color
            card.spawn((
                Text::new(spec.name.to_string()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(element_color(spec.element)),
            ));

            // HP bar container (background)
            card.spawn((
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                BorderRadius::all(Val::Px(2.0)),
            ))
            .with_children(|bar_bg| {
                // HP bar fill
                let bar_color = if spec.hp_pct > 0.5 {
                    Color::srgb(0.267, 0.8, 0.267) // green
                } else if spec.hp_pct > 0.25 {
                    Color::srgb(0.8, 0.8, 0.267) // yellow
                } else {
                    Color::srgb(0.8, 0.267, 0.267) // red
                };
                bar_bg.spawn((
                    Node {
                        width: Val::Percent(spec.hp_pct * 100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(bar_color),
                    BorderRadius::all(Val::Px(2.0)),
                    HpBarFill {
                        side: spec.side,
                        index: spec.index,
                    },
                ));
            });

            // HP text
            let hp_str = format!("{}/{}", spec.current_hp, spec.max_hp);
            card.spawn((
                Text::new(hp_str),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Crit counter (player units only)
            if matches!(spec.side, HudSide::Player) {
                let crit_str = format!("{}/10", spec.crit_counter);
                card.spawn((
                    Text::new(crit_str),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.667, 0.267)),
                    CritCounterText { index: spec.index },
                ));
            }
        });
}
