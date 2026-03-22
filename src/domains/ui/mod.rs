//! UI domain — visual layer for Vale Village v3.
//! Renders battle scenes, HUD, menus via Bevy 2D.
//! Imports from crate::shared, never redefines types locally.

pub mod animation;
pub mod app_state;
pub mod battle_scene;
pub mod dialogue_screen;
pub mod dungeon_screen;
pub mod gameover_screen;
pub mod hud;
pub mod menu_screen;
pub mod planning;
pub mod plugin;
pub mod puzzle_screen;
pub mod save_load_screen;
pub mod screenshot;
pub mod shop_screen;
pub mod title_screen;
pub mod town_screen;
pub mod ui_helpers;
pub mod world_map_screen;
