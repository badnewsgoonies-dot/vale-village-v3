// menu domain — Wave 2
#![allow(dead_code)]
//! Menu domain — text formatting for all menu screens.
//!
//! Pure formatting: each `show_*` function returns `Vec<String>` lines
//! ready for the caller to print. No I/O happens here.

use crate::shared::MenuScreen;
use std::collections::HashMap;

// ── MenuState ────────────────────────────────────────────────────────

/// Runtime state for the menu system: which screen is active and the
/// cursor position per screen.
#[derive(Debug, Clone)]
pub struct MenuState {
    pub current: MenuScreen,
    pub selected: HashMap<MenuScreen, usize>,
}

impl MenuState {
    pub fn new(initial: MenuScreen) -> Self {
        MenuState {
            current: initial,
            selected: HashMap::new(),
        }
    }

    /// Move the cursor for the current screen, clamping to `max`.
    pub fn select(&mut self, index: usize) {
        self.selected.insert(self.current, index);
    }

    pub fn selected_index(&self) -> usize {
        self.selected.get(&self.current).copied().unwrap_or(0)
    }

    pub fn switch_to(&mut self, screen: MenuScreen) {
        self.current = screen;
    }
}

// ── Summary structs (display-ready, decoupled from game data types) ──

/// Lightweight party member view for the menu layer.
#[derive(Debug, Clone)]
pub struct UnitSummary {
    pub name: String,
    pub level: u8,
    pub hp_current: u16,
    pub hp_max: u16,
    pub element: String,
}

/// Lightweight equipment view.
#[derive(Debug, Clone)]
pub struct EquipmentSummary {
    pub name: String,
    pub slot: String,
    pub tier: u8,
    pub equipped: bool,
}

/// Djinn state as the menu sees it.
#[derive(Debug, Clone, PartialEq)]
pub enum DjinnMenuState {
    Set,
    Standby,
    Recovery,
}

impl DjinnMenuState {
    fn label(&self) -> &'static str {
        match self {
            DjinnMenuState::Set => "SET",
            DjinnMenuState::Standby => "STANDBY",
            DjinnMenuState::Recovery => "RECOVERY",
        }
    }
}

/// Lightweight djinn view.
#[derive(Debug, Clone)]
pub struct DjinnSummary {
    pub name: String,
    pub element: String,
    pub state: DjinnMenuState,
    pub tier: u8,
}

/// Lightweight item view.
#[derive(Debug, Clone)]
pub struct ItemSummary {
    pub name: String,
    pub count: u8,
    pub description: String,
}

/// Quest log entry for display.
#[derive(Debug, Clone)]
pub struct QuestLogEntry {
    pub name: String,
    pub description: String,
    pub completed: bool,
}

// ── Formatting functions ─────────────────────────────────────────────

/// Format the party list. Each unit gets a line:
/// `[slot] Name   Lv.XX   HP: cur/max   Element`
pub fn show_party(units: &[UnitSummary]) -> Vec<String> {
    let mut lines = vec!["=== PARTY ===".to_string()];
    if units.is_empty() {
        lines.push("  (no party members)".to_string());
        return lines;
    }
    for (i, unit) in units.iter().enumerate() {
        lines.push(format!(
            "  [{}] {:<12} Lv.{:>2}   HP: {:>4}/{:>4}   {}",
            i + 1,
            unit.name,
            unit.level,
            unit.hp_current,
            unit.hp_max,
            unit.element,
        ));
    }
    lines
}

/// Format equipment for a unit: equipped items first, then available.
pub fn show_equipment(unit: &UnitSummary, equipment: &[EquipmentSummary]) -> Vec<String> {
    let mut lines = vec![format!("=== EQUIPMENT — {} ===", unit.name)];

    let equipped: Vec<&EquipmentSummary> = equipment.iter().filter(|e| e.equipped).collect();
    let available: Vec<&EquipmentSummary> = equipment.iter().filter(|e| !e.equipped).collect();

    lines.push("  Equipped:".to_string());
    if equipped.is_empty() {
        lines.push("    (nothing equipped)".to_string());
    } else {
        for item in &equipped {
            lines.push(format!(
                "    [{:>6}] Tier {} — {}",
                item.slot, item.tier, item.name
            ));
        }
    }

    lines.push("  Available:".to_string());
    if available.is_empty() {
        lines.push("    (no other equipment)".to_string());
    } else {
        for item in &available {
            lines.push(format!(
                "    [{:>6}] Tier {} — {}",
                item.slot, item.tier, item.name
            ));
        }
    }
    lines
}

/// Format the djinn list with state and element.
pub fn show_djinn(djinn_list: &[DjinnSummary]) -> Vec<String> {
    let mut lines = vec!["=== DJINN ===".to_string()];
    if djinn_list.is_empty() {
        lines.push("  (no djinn)".to_string());
        return lines;
    }
    for djinn in djinn_list {
        lines.push(format!(
            "  {:<14} [{:>8}]  Tier {}  {}",
            djinn.name,
            djinn.state.label(),
            djinn.tier,
            djinn.element,
        ));
    }
    lines
}

/// Format the item inventory with counts.
pub fn show_items(inventory: &[ItemSummary]) -> Vec<String> {
    let mut lines = vec!["=== ITEMS ===".to_string()];
    if inventory.is_empty() {
        lines.push("  (inventory empty)".to_string());
        return lines;
    }
    for item in inventory {
        lines.push(format!(
            "  {:<18} x{:<3}  {}",
            item.name, item.count, item.description,
        ));
    }
    lines
}

/// Format the quest log — active quests first, then completed.
pub fn show_quest_log(quests: &[QuestLogEntry]) -> Vec<String> {
    let mut lines = vec!["=== QUEST LOG ===".to_string()];

    let active: Vec<&QuestLogEntry> = quests.iter().filter(|q| !q.completed).collect();
    let completed: Vec<&QuestLogEntry> = quests.iter().filter(|q| q.completed).collect();

    lines.push("  Active:".to_string());
    if active.is_empty() {
        lines.push("    (no active quests)".to_string());
    } else {
        for quest in &active {
            lines.push(format!("    [ ] {}", quest.name));
            lines.push(format!("        {}", quest.description));
        }
    }

    lines.push("  Completed:".to_string());
    if completed.is_empty() {
        lines.push("    (none yet)".to_string());
    } else {
        for quest in &completed {
            lines.push(format!("    [x] {}", quest.name));
            lines.push(format!("        {}", quest.description));
        }
    }
    lines
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_units() -> Vec<UnitSummary> {
        vec![
            UnitSummary {
                name: "Isaac".to_string(),
                level: 10,
                hp_current: 180,
                hp_max: 200,
                element: "Venus".to_string(),
            },
            UnitSummary {
                name: "Garet".to_string(),
                level: 9,
                hp_current: 150,
                hp_max: 220,
                element: "Mars".to_string(),
            },
        ]
    }

    #[test]
    fn test_show_party_header() {
        let lines = show_party(&sample_units());
        assert_eq!(lines[0], "=== PARTY ===");
    }

    #[test]
    fn test_show_party_contains_name_and_level() {
        let lines = show_party(&sample_units());
        assert!(lines[1].contains("Isaac"));
        assert!(lines[1].contains("10"));
        assert!(lines[1].contains("Venus"));
        assert!(lines[1].contains("180"));
        assert!(lines[1].contains("200"));
    }

    #[test]
    fn test_show_party_empty() {
        let lines = show_party(&[]);
        assert!(lines.iter().any(|l| l.contains("no party members")));
    }

    #[test]
    fn test_show_equipment_header_contains_unit_name() {
        let unit = &sample_units()[0];
        let equipment = vec![
            EquipmentSummary {
                name: "Broad Sword".to_string(),
                slot: "Weapon".to_string(),
                tier: 2,
                equipped: true,
            },
            EquipmentSummary {
                name: "Iron Shield".to_string(),
                slot: "Shield".to_string(),
                tier: 1,
                equipped: false,
            },
        ];
        let lines = show_equipment(unit, &equipment);
        assert!(lines[0].contains("Isaac"));
        let equipped_section: Vec<&String> = lines.iter().filter(|l| l.contains("Broad Sword")).collect();
        assert!(!equipped_section.is_empty());
        let available_section: Vec<&String> = lines.iter().filter(|l| l.contains("Iron Shield")).collect();
        assert!(!available_section.is_empty());
    }

    #[test]
    fn test_show_equipment_empty() {
        let unit = &sample_units()[0];
        let lines = show_equipment(unit, &[]);
        assert!(lines.iter().any(|l| l.contains("nothing equipped")));
        assert!(lines.iter().any(|l| l.contains("no other equipment")));
    }

    #[test]
    fn test_show_djinn_states() {
        let djinn_list = vec![
            DjinnSummary {
                name: "Flint".to_string(),
                element: "Venus".to_string(),
                state: DjinnMenuState::Set,
                tier: 1,
            },
            DjinnSummary {
                name: "Forge".to_string(),
                element: "Mars".to_string(),
                state: DjinnMenuState::Standby,
                tier: 2,
            },
            DjinnSummary {
                name: "Fizz".to_string(),
                element: "Mercury".to_string(),
                state: DjinnMenuState::Recovery,
                tier: 1,
            },
        ];
        let lines = show_djinn(&djinn_list);
        assert_eq!(lines[0], "=== DJINN ===");
        assert!(lines[1].contains("Flint") && lines[1].contains("SET"));
        assert!(lines[2].contains("Forge") && lines[2].contains("STANDBY"));
        assert!(lines[3].contains("Fizz") && lines[3].contains("RECOVERY"));
    }

    #[test]
    fn test_show_djinn_empty() {
        let lines = show_djinn(&[]);
        assert!(lines.iter().any(|l| l.contains("no djinn")));
    }

    #[test]
    fn test_show_items_with_counts() {
        let inventory = vec![
            ItemSummary {
                name: "Herb".to_string(),
                count: 5,
                description: "Restores 50 HP".to_string(),
            },
            ItemSummary {
                name: "Antidote".to_string(),
                count: 2,
                description: "Cures poison".to_string(),
            },
        ];
        let lines = show_items(&inventory);
        assert_eq!(lines[0], "=== ITEMS ===");
        assert!(lines[1].contains("Herb") && lines[1].contains("5"));
        assert!(lines[2].contains("Antidote") && lines[2].contains("2"));
    }

    #[test]
    fn test_show_items_empty() {
        let lines = show_items(&[]);
        assert!(lines.iter().any(|l| l.contains("inventory empty")));
    }

    #[test]
    fn test_show_quest_log_active_and_completed() {
        let quests = vec![
            QuestLogEntry {
                name: "Find the Crystal".to_string(),
                description: "Locate the lost crystal in the cave.".to_string(),
                completed: false,
            },
            QuestLogEntry {
                name: "Defeat the Slime".to_string(),
                description: "Clear the slimes from the farm.".to_string(),
                completed: true,
            },
        ];
        let lines = show_quest_log(&quests);
        assert_eq!(lines[0], "=== QUEST LOG ===");
        let active_quest: Vec<&String> = lines.iter().filter(|l| l.contains("Find the Crystal")).collect();
        assert!(!active_quest.is_empty());
        assert!(active_quest[0].contains("[ ]"));
        let done_quest: Vec<&String> = lines.iter().filter(|l| l.contains("Defeat the Slime")).collect();
        assert!(!done_quest.is_empty());
        assert!(done_quest[0].contains("[x]"));
    }

    #[test]
    fn test_show_quest_log_empty() {
        let lines = show_quest_log(&[]);
        assert!(lines.iter().any(|l| l.contains("no active quests")));
        assert!(lines.iter().any(|l| l.contains("none yet")));
    }

    #[test]
    fn test_menu_state_switch_and_select() {
        let mut state = MenuState::new(MenuScreen::Party);
        assert_eq!(state.current, MenuScreen::Party);
        assert_eq!(state.selected_index(), 0);

        state.select(2);
        assert_eq!(state.selected_index(), 2);

        state.switch_to(MenuScreen::Items);
        assert_eq!(state.current, MenuScreen::Items);
        assert_eq!(state.selected_index(), 0); // fresh cursor for new screen

        state.switch_to(MenuScreen::Party);
        assert_eq!(state.selected_index(), 2); // remembered
    }
}
