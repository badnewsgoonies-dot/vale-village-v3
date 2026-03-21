//! Tuning values — mutable coefficients, NOT frozen.
use crate::shared::bounded_types::{
    Level, ManaCost, MaxBuffStacks, MaxEquippedDjinn, MaxPartySize,
};
use crate::shared::CombatConfig;

pub fn default_combat_config() -> CombatConfig {
    CombatConfig {
        physical_def_multiplier: 0.5,
        psynergy_def_multiplier: 0.3,
        crit_threshold: 10,
        crit_multiplier: 2.0,
        mana_gain_per_hit: ManaCost::new(1),
        mana_resets_each_round: true,
        max_party_size: MaxPartySize::new(4),
        max_equipped_djinn: MaxEquippedDjinn::new(3),
        max_level: Level::new(20),
        max_buff_stacks: MaxBuffStacks::new(3),
        djinn_recovery_start_delay: 1,
        djinn_recovery_per_turn: 1,
    }
}
