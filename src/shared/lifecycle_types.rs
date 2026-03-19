pub mod equipment {
    use ironclad::game_lifecycle;
    #[game_lifecycle(Basic -> Bronze -> Iron -> Steel -> Silver -> Mythril -> Legendary -> Artifact)]
    pub struct EquipmentProgression;
}

pub mod difficulty {
    use ironclad::game_lifecycle;
    #[game_lifecycle(Easy -> Medium -> Hard -> Boss)]
    pub struct DifficultyProgression;
}

pub mod battle {
    use ironclad::game_lifecycle;
    #[game_lifecycle(Planning -> Execution -> RoundEnd)]
    pub struct BattlePhaseProgression;
}
