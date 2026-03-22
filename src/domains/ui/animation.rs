//! Animation systems — floating damage numbers, status icons, battle event playback.
//! Wave 5: visual feedback for battle execution events.
#![allow(dead_code)]

use bevy::prelude::*;

use crate::domains::battle_engine::BattleEvent;
use crate::domains::sprite_loader::SpriteRegistry;
use crate::shared::{DamageType, Element, Side, StatusEffectType, TargetRef};

use super::battle_scene::{EnemyUnit, PlayerUnit, SpriteSwapTimer, UnitSpriteSet};
use super::planning::{PlanningMode, PlanningState, PostPlaybackMode};
use super::plugin::{BattleRes, GameDataRes};

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

/// Projectile flying from attacker to target.
#[derive(Component)]
pub struct ProjectileAnim {
    pub start: Vec3,
    pub end: Vec3,
    pub timer: Timer,
}

/// Impact effect that scales up and fades.
#[derive(Component)]
pub struct ImpactAnim {
    pub timer: Timer,
}

/// Hit stop — freezes the event queue for a brief moment at impact.
#[derive(Resource, Default)]
pub struct HitStop {
    pub timer: Option<Timer>,
}

/// Knockback — pushes a unit backward then springs back.
#[derive(Component)]
pub struct Knockback {
    pub timer: Timer,
    pub offset: f32,
    pub direction: f32, // 1.0 = right (enemy hit), -1.0 = left (player hit)
    pub original_x: f32,
}

/// Afterimage — fading copy of a sprite at a previous position.
#[derive(Component)]
pub struct Afterimage {
    pub timer: Timer,
}

// ── Event playback resource ────────────────────────────────────────

/// Queued events waiting to be visualized.
#[derive(Resource, Default)]
pub struct EventQueue {
    pub events: Vec<BattleEvent>,
    pub current_index: usize,
    pub event_timer: Timer,
    pub settle_timer: Option<Timer>,
    pub playing: bool,
}

#[derive(Clone, Copy)]
pub struct PlaybackProgress {
    pub shown: usize,
    pub total: usize,
    pub settling: bool,
}

pub fn playback_progress(queue: &EventQueue) -> PlaybackProgress {
    PlaybackProgress {
        shown: queue.current_index.min(queue.events.len()),
        total: queue.events.len(),
        settling: queue.settle_timer.is_some(),
    }
}

// ── Systems ────────────────────────────────────────────────────────

/// Check if there are new events to play from the planning state.
pub fn check_for_new_events(mut queue: ResMut<EventQueue>, state: Res<PlanningState>) {
    if state.mode == PlanningMode::Executing
        && !state.last_events.is_empty()
        && !queue.playing
        && queue.events.is_empty()
    {
        queue.events = state.last_events.clone();
        queue.current_index = 0;
        queue.event_timer = Timer::from_seconds(0.6, TimerMode::Repeating);
        queue.settle_timer = None;
        queue.playing = true;
    }
}

/// Play events from the queue one at a time.
pub fn play_event_queue(
    mut commands: Commands,
    time: Res<Time>,
    mut queue: ResMut<EventQueue>,
    mut state: ResMut<PlanningState>,
    mut hit_stop: ResMut<HitStop>,
    player_q: Query<(&Transform, &PlayerUnit)>,
    enemy_q: Query<(&Transform, &EnemyUnit)>,
    mut sprite_q: Query<(
        Entity,
        &mut Sprite,
        &UnitSpriteSet,
        Option<&PlayerUnit>,
        Option<&EnemyUnit>,
    )>,
    sprite_registry: Res<SpriteRegistry>,
    battle: Res<BattleRes>,
    game_data: Res<GameDataRes>,
) {
    if !queue.playing {
        return;
    }

    // Hit stop: freeze everything for a few frames
    if let Some(hs_timer) = hit_stop.timer.as_mut() {
        hs_timer.tick(time.delta());
        if hs_timer.finished() {
            hit_stop.timer = None;
        }
        return; // Frozen — skip all playback
    }

    if queue.current_index < queue.events.len() {
        queue.event_timer.tick(time.delta());

        if queue.event_timer.just_finished() {
            let event = queue.events[queue.current_index].clone();
            spawn_event_visual(&mut commands, &event, &player_q, &enemy_q);
            apply_sprite_swap(&mut commands, &event, &mut sprite_q);
            spawn_projectile_effect(
                &mut commands,
                &event,
                &player_q,
                &enemy_q,
                &sprite_registry,
                &battle.0,
                &game_data.0,
            );
            spawn_afterimages(&mut commands, &event, &player_q, &enemy_q);

            // Hit stop on damage events — freeze 50ms at moment of contact
            if matches!(&event, BattleEvent::DamageDealt(d) if d.is_crit) {
                hit_stop.timer = Some(Timer::from_seconds(0.08, TimerMode::Once));
            } else if matches!(&event, BattleEvent::DamageDealt(_)) {
                hit_stop.timer = Some(Timer::from_seconds(0.05, TimerMode::Once));
            }

            queue.current_index += 1;

            // Variable timing: set next event delay based on what just played
            let next_delay = if queue.current_index < queue.events.len() {
                event_delay(&queue.events[queue.current_index])
            } else {
                0.6
            };
            queue.event_timer = Timer::from_seconds(next_delay, TimerMode::Repeating);

            if queue.current_index >= queue.events.len() {
                queue.settle_timer = Some(Timer::from_seconds(0.8, TimerMode::Once));
            }
        }
        return;
    }

    if let Some(settle_timer) = queue.settle_timer.as_mut() {
        settle_timer.tick(time.delta());

        if settle_timer.finished() {
            queue.playing = false;
            queue.current_index = 0;
            queue.events.clear();
            queue.settle_timer = None;

            if state.mode == PlanningMode::Executing {
                state.mode = match state
                    .post_playback_mode
                    .take()
                    .unwrap_or(PostPlaybackMode::RoundComplete)
                {
                    PostPlaybackMode::RoundComplete => PlanningMode::RoundComplete,
                    PostPlaybackMode::BattleOver => PlanningMode::BattleOver,
                };
            }
        }
    }
}

/// Variable timing per event type — slow-fast-slow pacing.
fn event_delay(event: &BattleEvent) -> f32 {
    match event {
        BattleEvent::DamageDealt(d) => {
            if d.is_crit {
                0.55
            } else {
                0.45
            }
        }
        BattleEvent::HealingDone(_) => 0.6,
        BattleEvent::UnitDefeated(_) => 0.7,
        BattleEvent::StatusApplied(_) => 0.35,
        BattleEvent::CritTriggered(_, _) => 0.3,
        BattleEvent::BarrierBlocked(_) => 0.3,
        BattleEvent::EnemyAbilityUsed { .. } => 0.4,
        _ => 0.3, // round markers, mana changes — fast
    }
}

/// Swap unit sprites based on battle events (attacker → attack pose, target → hit pose).
fn apply_sprite_swap(
    commands: &mut Commands,
    event: &BattleEvent,
    sprite_q: &mut Query<(
        Entity,
        &mut Sprite,
        &UnitSpriteSet,
        Option<&PlayerUnit>,
        Option<&EnemyUnit>,
    )>,
) {
    match event {
        BattleEvent::DamageDealt(dmg) => {
            swap_unit_sprite(
                commands,
                sprite_q,
                dmg.source,
                SpriteSwapKind::Attack,
                false,
            );
            swap_unit_sprite(
                commands,
                sprite_q,
                dmg.target,
                SpriteSwapKind::Hit,
                dmg.is_crit,
            );
        }
        BattleEvent::EnemyAbilityUsed { actor, .. } => {
            swap_unit_sprite(commands, sprite_q, *actor, SpriteSwapKind::Attack, false);
        }
        _ => {}
    }
}

enum SpriteSwapKind {
    Attack,
    Hit,
}

fn swap_unit_sprite(
    commands: &mut Commands,
    sprite_q: &mut Query<(
        Entity,
        &mut Sprite,
        &UnitSpriteSet,
        Option<&PlayerUnit>,
        Option<&EnemyUnit>,
    )>,
    target: TargetRef,
    kind: SpriteSwapKind,
    is_crit: bool,
) {
    for (entity, mut sprite, sprite_set, player, enemy) in sprite_q.iter_mut() {
        let matches = match target.side {
            Side::Player => player.is_some_and(|p| p.index == target.index),
            Side::Enemy => enemy.is_some_and(|e| e.index == target.index),
        };
        if matches {
            let new_handle = match kind {
                SpriteSwapKind::Attack => sprite_set.attack.clone(),
                SpriteSwapKind::Hit => sprite_set.hit.clone(),
            };
            sprite.image = new_handle;
            commands.entity(entity).insert(SpriteSwapTimer {
                timer: Timer::from_seconds(0.4, TimerMode::Once),
                idle_handle: sprite_set.idle.clone(),
            });
            // Knockback on hit
            if matches!(kind, SpriteSwapKind::Hit) {
                let direction = match target.side {
                    Side::Player => -1.0,
                    Side::Enemy => 1.0,
                };
                let offset = if is_crit { 14.0 } else { 8.0 };
                commands.entity(entity).insert(Knockback {
                    timer: Timer::from_seconds(0.25, TimerMode::Once),
                    offset,
                    direction,
                    original_x: 0.0, // Will be set by animate_knockbacks on first tick
                });
            }
            break;
        }
    }
}

/// Revert sprite swaps back to idle when their timers expire.
pub fn revert_sprite_swaps(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut SpriteSwapTimer)>,
) {
    for (entity, mut sprite, mut swap) in query.iter_mut() {
        swap.timer.tick(time.delta());
        if swap.timer.finished() {
            sprite.image = swap.idle_handle.clone();
            commands.entity(entity).remove::<SpriteSwapTimer>();
        }
    }
}

// ── Knockback & Afterimage Effects ──────────────────────────────────

/// Animate knockbacks — push out then spring back.
pub fn animate_knockbacks(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Knockback)>,
) {
    for (entity, mut transform, mut kb) in query.iter_mut() {
        // Capture original position on first tick
        if kb.original_x == 0.0 && kb.timer.elapsed_secs() == 0.0 {
            kb.original_x = transform.translation.x;
        }
        kb.timer.tick(time.delta());
        let t = kb.timer.fraction();

        // Ease out: fast push, slow recovery
        let displacement = if t < 0.3 {
            let push_t = t / 0.3;
            kb.offset * kb.direction * push_t
        } else {
            let recover_t = (t - 0.3) / 0.7;
            kb.offset * kb.direction * (1.0 - recover_t * recover_t)
        };

        transform.translation.x = kb.original_x + displacement;

        if kb.timer.finished() {
            transform.translation.x = kb.original_x;
            commands.entity(entity).remove::<Knockback>();
        }
    }
}

/// Spawn afterimage trails on attackers.
fn spawn_afterimages(
    commands: &mut Commands,
    event: &BattleEvent,
    player_q: &Query<(&Transform, &PlayerUnit)>,
    enemy_q: &Query<(&Transform, &EnemyUnit)>,
) {
    let source = match event {
        BattleEvent::DamageDealt(dmg) => dmg.source,
        BattleEvent::EnemyAbilityUsed { actor, .. } => *actor,
        _ => return,
    };

    let Some(pos) = get_unit_position(source, player_q, enemy_q) else {
        return;
    };

    // Spawn 2 afterimage ghosts at slightly offset positions
    for i in 0..2 {
        let offset_x = -(i as f32 + 1.0) * 6.0;
        let alpha = 0.3 - i as f32 * 0.1;
        commands.spawn((
            Sprite::from_color(Color::srgba(0.8, 0.8, 1.0, alpha), Vec2::new(48.0, 64.0)),
            Transform::from_translation(pos + Vec3::new(offset_x, 0.0, -0.5)),
            Afterimage {
                timer: Timer::from_seconds(0.2 + i as f32 * 0.05, TimerMode::Once),
            },
        ));
    }
}

/// Fade out and despawn afterimages.
pub fn animate_afterimages(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut Afterimage)>,
) {
    for (entity, mut sprite, mut after) in query.iter_mut() {
        after.timer.tick(time.delta());
        let t = after.timer.fraction();
        sprite.color = Color::srgba(0.8, 0.8, 1.0, 0.3 * (1.0 - t));

        if after.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Projectile & Impact Effects ─────────────────────────────────────

/// Look up unit element from Battle + GameData by TargetRef.
fn element_for_unit(
    target: TargetRef,
    battle: &crate::domains::battle_engine::Battle,
    game_data: &crate::domains::data_loader::GameData,
) -> Element {
    use crate::shared::{EnemyId, UnitId};
    let id = match target.side {
        Side::Player => battle
            .player_units
            .get(target.index as usize)
            .map(|u| u.unit.id.as_str()),
        Side::Enemy => battle
            .enemies
            .get(target.index as usize)
            .map(|u| u.unit.id.as_str()),
    };
    let Some(id) = id else {
        return Element::Venus;
    };
    if let Some(unit_def) = game_data.units.get(&UnitId(id.to_string())) {
        return unit_def.element;
    }
    if let Some(enemy_def) = game_data.enemies.get(&EnemyId(id.to_string())) {
        return enemy_def.element;
    }
    Element::Venus
}

fn element_key(element: Element) -> &'static str {
    match element {
        Element::Venus => "venus",
        Element::Mars => "mars",
        Element::Mercury => "mercury",
        Element::Jupiter => "jupiter",
    }
}

/// Spawn a projectile sprite that flies from source to target on DamageDealt (Psynergy only).
fn spawn_projectile_effect(
    commands: &mut Commands,
    event: &BattleEvent,
    player_q: &Query<(&Transform, &PlayerUnit)>,
    enemy_q: &Query<(&Transform, &EnemyUnit)>,
    sprite_registry: &SpriteRegistry,
    battle: &crate::domains::battle_engine::Battle,
    game_data: &crate::domains::data_loader::GameData,
) {
    let BattleEvent::DamageDealt(dmg) = event else {
        return;
    };
    // Only psynergy gets a projectile; physical attacks use the sprite swap only
    if dmg.damage_type != DamageType::Psynergy {
        return;
    }

    let Some(source_pos) = get_unit_position(dmg.source, player_q, enemy_q) else {
        return;
    };
    let Some(target_pos) = get_unit_position(dmg.target, player_q, enemy_q) else {
        return;
    };

    let element = element_for_unit(dmg.source, battle, game_data);
    let key = element_key(element);

    let Some(proj_handle) = sprite_registry.get_effect_projectile(key) else {
        return;
    };

    // Spawn projectile entity — BIG and BRIGHT
    commands.spawn((
        Sprite {
            image: proj_handle,
            custom_size: Some(Vec2::new(64.0, 64.0)),
            ..default()
        },
        Transform::from_translation(source_pos + Vec3::new(0.0, 0.0, 5.0)),
        ProjectileAnim {
            start: source_pos,
            end: target_pos,
            timer: Timer::from_seconds(0.30, TimerMode::Once),
        },
    ));

    // Pre-spawn impact — HUGE, invisible until projectile arrives
    if let Some(impact_handle) = sprite_registry.get_effect_impact(key) {
        commands.spawn((
            Sprite {
                image: impact_handle,
                custom_size: Some(Vec2::new(96.0, 96.0)),
                color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                ..default()
            },
            Transform::from_translation(target_pos + Vec3::new(0.0, 0.0, 6.0)),
            ImpactAnim {
                timer: Timer::from_seconds(99.0, TimerMode::Once),
            },
        ));
    }
}

/// Animate projectiles: lerp with dramatic arc, scale up as they fly, despawn on arrival.
pub fn animate_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut proj_q: Query<(Entity, &mut Transform, &mut Sprite, &mut ProjectileAnim)>,
    mut impact_q: Query<
        (Entity, &mut Sprite, &mut ImpactAnim, &Transform),
        Without<ProjectileAnim>,
    >,
    camera_q: Query<Entity, With<Camera2d>>,
) {
    for (entity, mut transform, mut sprite, mut proj) in proj_q.iter_mut() {
        proj.timer.tick(time.delta());
        let t = proj.timer.fraction();

        // Lerp with dramatic high arc
        let pos = proj.start.lerp(proj.end, t);
        let arc_y = (t * std::f32::consts::PI).sin() * -60.0;
        transform.translation = Vec3::new(pos.x, pos.y + arc_y, pos.z);

        // Spin fast
        transform.rotation = Quat::from_rotation_z(t * std::f32::consts::PI * 6.0);

        // Scale up as it flies: 1.0 → 1.5 at peak → 1.0 at end
        let scale_boost = 1.0 + (t * std::f32::consts::PI).sin() * 0.5;
        transform.scale = Vec3::splat(scale_boost);

        // Brighten at peak
        let brightness = 1.0 + (t * std::f32::consts::PI).sin() * 0.3;
        sprite.color = Color::srgba(brightness, brightness, brightness, 1.0);

        if proj.timer.finished() {
            commands.entity(entity).despawn();

            // Trigger impact
            for (_ie, mut impact_sprite, mut impact, _it) in impact_q.iter_mut() {
                if impact.timer.remaining_secs() > 90.0 {
                    impact.timer = Timer::from_seconds(0.45, TimerMode::Once);
                    impact_sprite.color = Color::srgba(1.0, 1.0, 1.0, 1.0);
                    break;
                }
            }

            // Screen shake on impact
            if let Ok(cam_entity) = camera_q.get_single() {
                commands.entity(cam_entity).insert(ScreenShake {
                    timer: Timer::from_seconds(0.25, TimerMode::Once),
                    intensity: 8.0,
                    original_pos: None,
                });
            }

            // Screen flash on impact
            commands.spawn((
                Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.6), Vec2::new(2000.0, 2000.0)),
                Transform::from_xyz(0.0, 0.0, 50.0),
                ScreenFlash {
                    timer: Timer::from_seconds(0.15, TimerMode::Once),
                },
            ));
        }
    }
}

/// Animate impacts: BOLD scale up and fade out with glow.
pub fn animate_impacts(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut ImpactAnim)>,
) {
    for (entity, mut transform, mut sprite, mut impact) in query.iter_mut() {
        if impact.timer.remaining_secs() > 90.0 {
            continue;
        }
        impact.timer.tick(time.delta());
        let t = impact.timer.fraction();

        // Scale up from 0.3 to 3.5 — BIG explosion
        let scale = 0.3 + t * 3.2;
        transform.scale = Vec3::splat(scale);

        // Fade out in second half, stay bright in first half
        let alpha = if t < 0.3 { 1.0 } else { 1.0 - (t - 0.3) / 0.7 };
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);

        if impact.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Screen shake — applied to camera, decays over duration.
#[derive(Component)]
pub struct ScreenShake {
    pub timer: Timer,
    pub intensity: f32,
    pub original_pos: Option<Vec3>,
}

/// Screen flash — white overlay that fades quickly.
#[derive(Component)]
pub struct ScreenFlash {
    pub timer: Timer,
}

/// Shake the camera during impacts.
pub fn animate_screen_shake(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut ScreenShake), With<Camera2d>>,
) {
    for (entity, mut transform, mut shake) in query.iter_mut() {
        if shake.original_pos.is_none() {
            shake.original_pos = Some(transform.translation);
        }
        shake.timer.tick(time.delta());
        let t = shake.timer.fraction();
        let decay = 1.0 - t;
        let offset_x = (t * 47.0).sin() * shake.intensity * decay;
        let offset_y = (t * 73.0).cos() * shake.intensity * decay;

        if let Some(orig) = shake.original_pos {
            transform.translation = orig + Vec3::new(offset_x, offset_y, 0.0);
        }

        if shake.timer.finished() {
            if let Some(orig) = shake.original_pos {
                transform.translation = orig;
            }
            commands.entity(entity).remove::<ScreenShake>();
        }
    }
}

/// Fade out and despawn the white flash overlay.
pub fn animate_screen_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut ScreenFlash)>,
) {
    for (entity, mut sprite, mut flash) in query.iter_mut() {
        flash.timer.tick(time.delta());
        let t = flash.timer.fraction();
        sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.6 * (1.0 - t));

        if flash.timer.finished() {
            commands.entity(entity).despawn();
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
                    (Color::srgb(1.0, 0.84, 0.0), 28.0) // gold, BIG
                } else {
                    (Color::WHITE, 20.0)
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
                spawn_floating_label(commands, pos, "CRIT!", Color::srgb(1.0, 0.84, 0.0), 26.0);
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
                spawn_floating_label(commands, pos, "KO", Color::srgb(1.0, 0.2, 0.2), 32.0);
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
                let label = format!(
                    "{:?} → {:?}",
                    djinn_change.old_state, djinn_change.new_state
                );
                spawn_floating_label(commands, pos, &label, Color::srgb(0.8, 0.8, 0.267), 12.0);
            }
        }

        BattleEvent::EnemyAbilityUsed {
            actor,
            ability_name,
            ..
        } => {
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
    spawn_floating_label_at(
        commands,
        Vec3::new(pos.x, pos.y + 20.0, 10.0),
        &text,
        color,
        size,
    );
}

fn spawn_floating_label(commands: &mut Commands, pos: Vec3, label: &str, color: Color, size: f32) {
    spawn_floating_label_at(
        commands,
        Vec3::new(pos.x, pos.y + 20.0, 10.0),
        label,
        color,
        size,
    );
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
        StatusEffectType::Stun => Color::srgb(0.8, 0.8, 0.267), // yellow
        StatusEffectType::Null => Color::srgb(0.5, 0.5, 0.5),   // gray
        StatusEffectType::Incapacitate => Color::srgb(0.8, 0.267, 0.267), // red
        StatusEffectType::Burn => Color::srgb(0.8, 0.533, 0.267), // orange
        StatusEffectType::Poison => Color::srgb(0.533, 0.267, 0.8), // purple
        StatusEffectType::Freeze => Color::srgb(0.267, 0.667, 0.8), // light blue
    }
}
