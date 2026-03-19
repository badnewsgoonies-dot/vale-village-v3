use ironclad::game_value;

#[game_value(min = 1, max = 9999)]
pub struct Hp(pub u16);

#[game_value(min = 0, max = 9999)]
pub struct BaseStat(pub u16);

#[game_value(min = -999, max = 999)]
pub struct StatMod(pub i16);

#[game_value(min = 0, max = 999)]
pub struct GrowthRate(pub u16);

#[game_value(min = 0, max = 99)]
pub struct ManaCost(pub u8);

#[game_value(min = 0, max = 9999)]
pub struct BasePower(pub u16);

#[game_value(min = 1, max = 10)]
pub struct HitCount(pub u8);

#[game_value(min = 1, max = 99)]
pub struct Level(pub u8);

#[game_value(min = 0, max = 10)]
pub struct EffectDuration(pub u8);

#[game_value(min = 1, max = 4)]
pub struct DjinnTier(pub u8);

#[game_value(min = 0, max = 999999)]
pub struct Gold(pub u32);

impl Default for Gold {
    fn default() -> Self {
        Self::new_unchecked(0)
    }
}

#[game_value(min = 0, max = 999999)]
pub struct Xp(pub u32);

impl Default for Xp {
    fn default() -> Self {
        Self::new_unchecked(0)
    }
}

#[game_value(min = 0, max = 20)]
pub struct ManaPool(pub u8);

#[game_value(min = 0, max = 7)]
pub struct PartyIndex(pub u8);

#[game_value(min = 1, max = 8)]
pub struct MaxPartySize(pub u8);

#[game_value(min = 0, max = 8)]
pub struct MaxEquippedDjinn(pub u8);

#[game_value(min = 1, max = 10)]
pub struct MaxBuffStacks(pub u8);

// Default impls for types used in Default-deriving structs
impl Default for Hp {
    fn default() -> Self {
        Self::new_unchecked(1)
    }
}

impl Default for BaseStat {
    fn default() -> Self {
        Self::new_unchecked(0)
    }
}

impl Default for GrowthRate {
    fn default() -> Self {
        Self::new_unchecked(0)
    }
}

impl Default for EffectDuration {
    fn default() -> Self {
        Self::new_unchecked(0)
    }
}
