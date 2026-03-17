//! Animation systems — floating damage numbers, status icons, battle event playback.
//! Wave 5: visual feedback for battle execution events.
#![allow(dead_code)]

use bevy::prelude::*;

use crate::domains::battle_engine::BattleEvent;
use crate::shared::{Side, StatusEffectType, TargetRef};

use super::battle_scene::{EnemyUnit, PlayerUnit};
use super::planning::PlanningState;

// ── Components ─────────────────────────────────────────────────────

/// Floating text that drifts upward and fades out.
#[derive(Component)]
pub struct FloatingText {
    pub timer: Timer,
    pub start_y: f32,
    pub drift: f32, // total pixels to drift up
}

/// Status icon displayed on a unit.
#[derive(Component)]
pub struct StatusIcon {
    pub unit_ref: TargetRef,
    pub timer: Timer,
}

/// Flash overlay on a unit sprite.
#[derive(Component)]
pub struct FlashEffect {
    pub timer: Timer,
    pub original_color: Color,
}

/// Marker for defeated unit fade-out.
#[derive(Component)]
pub struct DefeatedFade {
    pub timer: Timer,
}

// ── Event playback resource ────────────────────────────────────────

/// Queued events waiting to be visualized.
#[derive(Resource, Default)]
pub struct EventQueue {
    pub events: Vec<BattleEvent>,
    pub current_index: usize,
    pub event_timer: Timer,
    pub playing: bool,
}

// ── Systems ────────────────────────────────────────────────────────

/// Check if there are new events to play from the planning state.
pub fn check_for_new_events(
    mut queue: ResMut<EventQueue>,
    state: Res<PlanningState>,
) {
    if state.is_changed() && !state.last_events.is_empty() && !queue.playing {
        queue.events = state.last_events.clone();
        queue.current_index = 0;
        queue.event_timer = Timer::from_seconds(0.6, TimerMode::Repeating);
        queue.playing = true;
    }
}

/// Play events from the queue one at a time.
pub fn play_event_queue(
    mut commands: Commands,
    time: Res<Time>,
    mut queue: ResMut<EventQueue>,
    player_q: Query<(&Transform, &PlayerUnit)>,
    enemy_q: Query<(&Transform, &EnemyUnit)>,
) {
    if !queue.playing {
        return;
    }

    queue.event_timer.tick(time.delta());

    if queue.event_timer.just_finished() && queue.current_index < queue.events.len() {
        let event = queue.events[queue.current_index].clone();
        spawn_event_visual(&mut commands, &event, &player_q, &enemy_q);
        queue.current_index += 1;

        if queue.current_index >= queue.events.len() {
            queue.playing = false;
        }
    }
}

/// Spawn visual effect for a single battle event.
fn spawn_event_visual(
    commands: &mut Commands,
    event: &BattleEvent,
    player_q: &Query<(&Transform, &PlayerUnit)>,
    enemy_q: &Query<(&Transform, &EnemyUnit)>,
) {
    match event {
        BattleEvent::DamageDealt(dmg) => {
            if let Some(pos) = get_unit_position(dmg.target, player_q, enemy_q) {
                let (color, size) = if dmg.is_crit {
                    (Color::srgb(1.0, 0.84, 0.0), 20.0) // gold, larger
                } else {
                    (Color::WHITE, 16.0)
                };
                spawn_floating_number(commands, pos, dmg.amount as i32, color, size);
            }
        }

        BattleEvent::HealingDone(heal) => {
            if let Some(pos) = get_unit_position(heal.target, player_q, enemy_q) {
                spawn_floating_number(
                    commands,
                    pos,
                    heal.amount as i32,
                    Color::srgb(0.267, 0.8, 0.267), // green
                    16.0,
                );
            }
        }

        BattleEvent::CritTriggered(unit_ref, _hit) => {
            if let Some(pos) = get_unit_position(*unit_ref, player_q, enemy_q) {
                spawn_floating_label(commands, pos, "CRIT!", Color::srgb(1.0, 0.84, 0.0), 18.0);
            }
        }

        BattleEvent::StatusApplied(status) => {
            if let Some(pos) = get_unit_position(status.target, player_q, enemy_q) {
                let label = status_label(status.effect.effect_type);
                let color = status_color(status.effect.effect_type);
                spawn_floating_label(commands, pos, label, color, 14.0);
            }
        }

        BattleEvent::BarrierBlocked(unit_ref) => {
            if let Some(pos) = get_unit_position(*unit_ref, player_q, enemy_q) {
                spawn_floating_label(
                    commands,
                    pos,
                    "BLOCKED",
                    Color::srgb(0.267, 0.533, 0.8),
                    14.0,
                );
            }
        }

        BattleEvent::UnitDefeated(defeated) => {
            if let Some(pos) = get_unit_position(defeated.unit, player_q, enemy_q) {
                spawn_floating_label(
                    commands,
                    pos,
                    "KO",
                    Color::srgb(0.8, 0.267, 0.267),
                    22.0,
                );
            }
        }

        BattleEvent::ManaChanged(mana) => {
            let diff = mana.new_value as i32 - mana.old_value as i32;
            if diff != 0 {
                let label = if diff > 0 {
                    format!("+{} mana", diff)
                } else {
                    format!("{} mana", diff)
                };
                let color = if diff > 0 {
                    Color::srgb(0.267, 0.533, 0.8)
                } else {
                    Color::srgb(0.8, 0.533, 0.267)
                };
                // Show at top center
                let pos = Vec3::new(0.0, 280.0, 10.0);
                spawn_floating_label_at(commands, pos, &label, color, 16.0);
            }
        }

        BattleEvent::DjinnChanged(djinn_change) => {
            if let Some(pos) = get_unit_position(djinn_change.unit, player_q, enemy_q) {
                let label = format!("{:?} → {:?}", djinn_change.old_state, djinn_change.new_state);
                spawn_floating_label(commands, pos, &label, Color::srgb(0.8, 0.8, 0.267), 12.0);
            }
        }

        BattleEvent::EnemyAbilityUsed { actor, ability_name, .. } => {
            if let Some(pos) = get_unit_position(*actor, player_q, enemy_q) {
                spawn_floating_label(
                    commands,
                    pos,
                    ability_name,
                    Color::srgb(0.8, 0.5, 0.5),
                    14.0,
                );
            }
        }

        // Round start/end don't need visuals (handled by planning state)
        BattleEvent::RoundStarted(_) | BattleEvent::RoundEnded(_) => {}
    }
}

/// Animate floating text — drift up and fade out.
pub fn animate_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FloatingText, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut float, mut transform, mut text_color) in query.iter_mut() {
        float.timer.tick(time.delta());

        let progress = float.timer.fraction();
        let y_offset = float.drift * progress;
        transform.translation.y = float.start_y + y_offset;

        // Fade out in last 30%
        let alpha = if progress > 0.7 {
            1.0 - (progress - 0.7) / 0.3
        } else {
            1.0
        };

        if let Color::Srgba(ref mut srgba) = text_color.0 {
            srgba.alpha = alpha;
        }

        if float.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────

/// Get the world position of a unit by its TargetRef.
fn get_unit_position(
    target: TargetRef,
    player_q: &Query<(&Transform, &PlayerUnit)>,
    enemy_q: &Query<(&Transform, &EnemyUnit)>,
) -> Option<Vec3> {
    match target.side {
        Side::Player => {
            for (transform, pu) in player_q.iter() {
                if pu.index == target.index {
                    return Some(transform.translation);
                }
            }
        }
        Side::Enemy => {
            for (transform, eu) in enemy_q.iter() {
                if eu.index == target.index {
                    return Some(transform.translation);
                }
            }
        }
    }
    None
}

fn spawn_floating_number(commands: &mut Commands, pos: Vec3, amount: i32, color: Color, size: f32) {
    let text = format!("{}", amount);
    spawn_floating_label_at(commands, Vec3::new(pos.x, pos.y + 20.0, 10.0), &text, color, size);
}

fn spawn_floating_label(commands: &mut Commands, pos: Vec3, label: &str, color: Color, size: f32) {
    spawn_floating_label_at(commands, Vec3::new(pos.x, pos.y + 20.0, 10.0), label, color, size);
}

fn spawn_floating_label_at(
    commands: &mut Commands,
    pos: Vec3,
    label: &str,
    color: Color,
    size: f32,
) {
    commands.spawn((
        Text2d::new(label.to_string()),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(pos),
        FloatingText {
            timer: Timer::from_seconds(0.8, TimerMode::Once),
            start_y: pos.y,
            drift: 40.0,
        },
    ));
}

fn status_label(effect_type: StatusEffectType) -> &'static str {
    match effect_type {
        StatusEffectType::Stun => "STUN",
        StatusEffectType::Null => "NULL",
        StatusEffectType::Incapacitate => "INCAP",
        StatusEffectType::Burn => "BURN",
        StatusEffectType::Poison => "POISON",
        StatusEffectType::Freeze => "FREEZE",
    }
}

fn status_color(effect_type: StatusEffectType) -> Color {
    match effect_type {
        StatusEffectType::Stun => Color::srgb(0.8, 0.8, 0.267),    // yellow
        StatusEffectType::Null => Color::srgb(0.5, 0.5, 0.5),      // gray
        StatusEffectType::Incapacitate => Color::srgb(0.8, 0.267, 0.267), // red
        StatusEffectType::Burn => Color::srgb(0.8, 0.533, 0.267),   // orange
        StatusEffectType::Poison => Color::srgb(0.533, 0.267, 0.8), // purple
        StatusEffectType::Freeze => Color::srgb(0.267, 0.667, 0.8), // light blue
    }
}
