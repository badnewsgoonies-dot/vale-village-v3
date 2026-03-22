//! AppState — Bevy state machine for top-level game screens.
//! Resource wrappers for SaveData and screen-context resources.
//!
//! Types imported from crate::shared and crate::domains — never redefined here.

use bevy::prelude::*;

use crate::domains::save::SaveData;
use crate::game_state::GameState;
use crate::shared::{ShopId, TownId};

// ── Bevy States enum ────────────────────────────────────────────────

/// Top-level application state driving screen transitions.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Title,
    WorldMap,
    Town,
    Shop,
    InBattle,
    PostBattle,
}

// ── Resource wrappers ───────────────────────────────────────────────

/// Bevy Resource wrapping the full game state (world map, quests, gold, etc.).
#[derive(Resource)]
pub struct GameStateRes(pub GameState);

/// Bevy Resource wrapping the active SaveData.
#[derive(Resource)]
pub struct SaveDataRes(pub SaveData);

/// Bevy Resource tracking which town the player is in.
#[derive(Resource)]
pub struct CurrentTown(pub TownId);

/// Bevy Resource tracking which shop the player is browsing.
#[derive(Resource)]
pub struct CurrentShop(pub ShopId);
