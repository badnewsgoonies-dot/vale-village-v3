#!/usr/bin/env python3
"""
Convert v2 JSON game data into RON files for Vale Village v3.

Reads from docs/data/*.json, writes to data/full/*.ron.
"""

import json
import os
import sys
from pathlib import Path

# Resolve project root relative to this script
SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
SRC_DIR = PROJECT_ROOT / "docs" / "data"
OUT_DIR = PROJECT_ROOT / "data" / "full"


# ── RON Helpers ──────────────────────────────────────────────────────

def ron_string(s):
    """Escape and quote a string for RON."""
    escaped = s.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def ron_option(value, formatter=None):
    """Format a RON Option<T>: None or Some(value)."""
    if value is None:
        return "None"
    if formatter:
        return f"Some({formatter(value)})"
    return f"Some({value})"


def ron_option_str(value):
    return ron_option(value, ron_string)


def ron_bool(value):
    return "true" if value else "false"


def ron_f32(value):
    """Format f32 — ensure decimal point."""
    if isinstance(value, int):
        return f"{value}.0"
    s = str(value)
    if "." not in s:
        s += ".0"
    return s


def ron_element(elem):
    """Convert element string to RON enum variant."""
    if elem is None:
        return None
    mapping = {
        "venus": "Venus",
        "mars": "Mars",
        "mercury": "Mercury",
        "jupiter": "Jupiter",
    }
    # "Neutral" and unknown elements map to None (no element)
    return mapping.get(elem.lower())


def ron_option_element(elem):
    """Element as Option<Element>."""
    val = ron_element(elem)
    if val is None:
        return "None"
    return f"Some({val})"


def indent(text, level=2):
    """Indent each line of text."""
    prefix = "    " * level
    return "\n".join(prefix + line for line in text.split("\n"))


# ── Ability Conversion ───────────────────────────────────────────────

def convert_category(type_str):
    """v2 type -> RON AbilityCategory."""
    mapping = {
        "physical": "Physical",
        "psynergy": "Psynergy",
        "healing": "Healing",
        "buff": "Buff",
        "debuff": "Debuff",
    }
    return mapping[type_str.lower()]


def convert_damage_type(type_str):
    """v2 type -> RON Option<DamageType>."""
    if type_str.lower() == "physical":
        return "Some(Physical)"
    elif type_str.lower() == "psynergy":
        return "Some(Psynergy)"
    return "None"


def convert_targets(targets_str):
    """v2 targets string -> RON TargetMode."""
    mapping = {
        "single-enemy": "SingleEnemy",
        "all-enemies": "AllEnemies",
        "single-ally": "SingleAlly",
        "all-allies": "AllAllies",
        "self": "SelfOnly",
    }
    return mapping[targets_str]


def convert_status_effect_type(type_str):
    """v2 status type -> RON StatusEffectType."""
    t = type_str.lower()
    if t == "paralyze":
        return "Stun"
    mapping = {
        "stun": "Stun",
        "null": "Null",
        "incapacitate": "Incapacitate",
        "burn": "Burn",
        "poison": "Poison",
        "freeze": "Freeze",
    }
    return mapping[t]


def convert_status_effect(se):
    """Convert v2 statusEffect object to RON StatusEffect."""
    if se is None:
        return "None"

    effect_type = convert_status_effect_type(se["type"])
    duration = se.get("duration", 1)

    # Determine burn_percent, poison_percent, freeze_threshold based on type
    burn_percent = "None"
    poison_percent = "None"
    freeze_threshold = "None"

    raw_type = se["type"].lower()
    if raw_type == "burn":
        burn_percent = f"Some({ron_f32(0.10)})"
    elif raw_type == "poison":
        poison_percent = f"Some({ron_f32(0.08)})"
    elif raw_type == "freeze":
        freeze_threshold = "Some(40)"

    return f"""Some((
            effect_type: {effect_type},
            duration: {duration},
            burn_percent: {burn_percent},
            poison_percent: {poison_percent},
            freeze_threshold: {freeze_threshold},
        ))"""


def convert_buff_effect(ability):
    """Convert v2 buffEffect to RON BuffEffect (for buff-type abilities)."""
    be = ability.get("buffEffect")
    if be is None:
        return "None"

    atk = be.get("atk") or 0
    def_ = be.get("def") or 0
    mag = be.get("mag") or 0
    spd = be.get("spd") or 0
    duration = be.get("duration") or 3  # Default to 3 if null

    # Check for shield charges inside buff
    shield = ability.get("shieldCharges")
    shield_str = ron_option(shield) if shield else "None"

    return f"""Some((
            stat_modifiers: (atk: {atk}, def: {def_}, mag: {mag}, spd: {spd}, hp: 0),
            duration: {duration},
            shield_charges: {shield_str},
            grant_immunity: false,
        ))"""


def convert_debuff_effect(ability):
    """Convert v2 buffEffect to RON DebuffEffect (for debuff-type abilities)."""
    be = ability.get("buffEffect")
    if be is None:
        return "None"

    atk = be.get("atk") or 0
    def_ = be.get("def") or 0
    mag = be.get("mag") or 0
    spd = be.get("spd") or 0
    duration = be.get("duration", 2)

    return f"""Some((
            stat_modifiers: (atk: {atk}, def: {def_}, mag: {mag}, spd: {spd}, hp: 0),
            duration: {duration},
        ))"""


def convert_immunity_types(types_list):
    """Convert v2 immunity type strings to RON StatusEffectType variants."""
    mapping = {
        "poison": "Poison",
        "burn": "Burn",
        "freeze": "Freeze",
        "stun": "Stun",
        "null": "Null",
        "paralyze": "Stun",
        "incapacitate": "Incapacitate",
    }
    return [mapping[t.lower()] for t in types_list]


def convert_grant_immunity(gi):
    """Convert v2 grantImmunity to RON Immunity."""
    if gi is None:
        return "None"

    all_val = ron_bool(gi.get("all", False))
    duration = gi.get("duration", 2)

    if gi.get("all", False):
        types_str = "[]"
    else:
        types_list = gi.get("types", [])
        types_ron = convert_immunity_types(types_list)
        types_str = "[" + ", ".join(types_ron) + "]"

    return f"""Some((
            all: {all_val},
            types: {types_str},
            duration: {duration},
        ))"""


def convert_cleanse_type(ct):
    """Convert v2 cleanse_type to RON CleanseType."""
    if ct is None:
        return "None"
    if ct == "all":
        return "Some(All)"
    if ct == "negative":
        return "Some(Negative)"
    if ct == "byType":
        # Default to burn for Mars-themed cleanse
        return "Some(ByType([Burn]))"
    return "None"


def convert_heal_over_time(hot):
    """Convert v2 healOverTime to RON HealOverTime."""
    if hot is None:
        return "None"
    amount = hot["amount"]
    duration = hot["duration"]
    return f"Some((amount: {amount}, duration: {duration}))"


def convert_ability(a):
    """Convert a single ability JSON object to RON AbilityDef string."""
    aid = a["id"]
    name = a["name"]
    category = convert_category(a["type"])
    damage_type = convert_damage_type(a["type"])
    element = ron_option_element(a.get("element"))
    mana_cost = a.get("manaCost", 0)
    base_power = a.get("basePower", 0)
    targets = convert_targets(a["targets"])
    unlock_level = a.get("unlockLevel", 1)
    hit_count = a.get("hitCount")
    if hit_count is None:
        hit_count = 1

    # Status effect (drop chance field)
    status_effect = convert_status_effect(a.get("statusEffect"))

    # Buff / Debuff effect
    if category == "Debuff":
        buff_effect = "None"
        debuff_effect = convert_debuff_effect(a)
    else:
        buff_effect = convert_buff_effect(a)
        debuff_effect = "None"

    # Shield charges - from top-level shieldCharges or barrierCharges
    shield_charges_val = a.get("shieldCharges") or a.get("barrierCharges")
    # If buff_effect already consumed shieldCharges for a buff, keep top-level too
    shield_charges = ron_option(shield_charges_val) if shield_charges_val else "None"

    # Shield duration - not in v2 data, default to None
    shield_duration = "None"

    # Heal over time
    heal_over_time = convert_heal_over_time(a.get("healOverTime"))

    # Grant immunity
    grant_immunity = convert_grant_immunity(a.get("grantImmunity"))

    # Cleanse
    cleanse = convert_cleanse_type(a.get("cleanse_type"))

    # Ignore defense percent
    idp = a.get("ignoreDefensePercent")
    ignore_defense_percent = ron_option(idp, ron_f32) if idp is not None else "None"

    # Splash damage percent
    sdp = a.get("splashDamagePercent")
    splash_damage_percent = ron_option(sdp, ron_f32) if sdp is not None else "None"

    # Chain damage
    cd = a.get("chainDamage")
    chain_damage = ron_bool(bool(cd))

    # Revive
    revive = ron_bool(bool(a.get("revive")))
    revive_hp_pct = a.get("reviveHPPercent")
    revive_hp_percent = ron_option(revive_hp_pct, ron_f32) if revive_hp_pct is not None else "None"

    return f"""    (
        id: AbilityId({ron_string(aid)}),
        name: {ron_string(name)},
        category: {category},
        damage_type: {damage_type},
        element: {element},
        mana_cost: {mana_cost},
        base_power: {base_power},
        targets: {targets},
        unlock_level: {unlock_level},
        hit_count: {hit_count},
        status_effect: {status_effect},
        buff_effect: {buff_effect},
        debuff_effect: {debuff_effect},
        shield_charges: {shield_charges},
        shield_duration: {shield_duration},
        heal_over_time: {heal_over_time},
        grant_immunity: {grant_immunity},
        cleanse: {cleanse},
        ignore_defense_percent: {ignore_defense_percent},
        splash_damage_percent: {splash_damage_percent},
        chain_damage: {chain_damage},
        revive: {revive},
        revive_hp_percent: {revive_hp_percent},
    )"""


# ── Enemy Conversion ─────────────────────────────────────────────────

def assign_enemy_abilities(enemy, all_abilities):
    """
    Assign abilities to an enemy based on element and level.
    Enemies in v2 only have abilityCount, not actual ability references.
    We assign based on element-matching abilities and generic abilities.
    """
    eid = enemy["id"]
    element = enemy.get("element", "Venus")
    level = enemy.get("level", 1)
    count = enemy.get("abilityCount", 1)

    # Always include strike
    assigned = ["strike"]

    # Build a pool of abilities for this enemy's element
    elem_abilities = []
    generic_abilities = []

    for a in all_abilities:
        a_elem = a.get("element")
        a_type = a.get("type", "physical")
        a_level = a.get("unlockLevel", 1)

        # Skip abilities that are too high level for this enemy
        if a_level > level + 5:
            continue
        # Skip healing/buff for most enemies (only for higher level)
        if a_type in ("healing",) and level < 5:
            continue

        if a_elem and a_elem.lower() == element.lower():
            elem_abilities.append(a["id"])
        elif a_elem is None and a_type in ("physical", "psynergy"):
            generic_abilities.append(a["id"])

    # Fill up to count
    pool = elem_abilities + generic_abilities
    # Remove strike if already in pool to avoid duplication
    pool = [x for x in pool if x != "strike"]

    for ability_id in pool:
        if len(assigned) >= count:
            break
        if ability_id not in assigned:
            assigned.append(ability_id)

    # Pad with strike if we don't have enough
    while len(assigned) < count:
        assigned.append("strike")

    # Deduplicate while preserving order
    seen = set()
    result = []
    for a in assigned:
        if a not in seen:
            seen.add(a)
            result.append(a)

    return result[:count]


def convert_enemy(e, abilities_list):
    """Convert a single enemy JSON object to RON EnemyDef string."""
    eid = e["id"]
    name = e["name"]
    element = ron_element(e["element"])
    level = e.get("level", 1)
    hp = e["hp"]
    atk = e["atk"]
    def_ = e["def"]
    mag = e["mag"]
    spd = e["spd"]
    xp = e.get("xpReward", 0)
    gold = e.get("goldReward", 0)

    ability_ids = assign_enemy_abilities(e, abilities_list)
    abilities_str = ", ".join(f'AbilityId({ron_string(a)})' for a in ability_ids)

    return f"""    (
        id: EnemyId({ron_string(eid)}),
        name: {ron_string(name)},
        element: {element},
        level: {level},
        stats: (hp: {hp}, atk: {atk}, def: {def_}, mag: {mag}, spd: {spd}),
        xp: {xp},
        gold: {gold},
        abilities: [{abilities_str}],
    )"""


# ── Equipment Conversion ─────────────────────────────────────────────

def convert_slot(slot_str):
    mapping = {
        "weapon": "Weapon",
        "helm": "Helm",
        "armor": "Armor",
        "boots": "Boots",
        "accessory": "Accessory",
    }
    return mapping[slot_str.lower()]


def convert_tier(tier_str):
    mapping = {
        "basic": "Basic",
        "bronze": "Bronze",
        "iron": "Iron",
        "steel": "Steel",
        "silver": "Silver",
        "mythril": "Mythril",
        "legendary": "Legendary",
        "artifact": "Artifact",
    }
    return mapping[tier_str.lower()]


def convert_equipment(eq):
    """Convert a single equipment JSON object to RON EquipmentDef string."""
    eid = eq["id"]
    name = eq["name"]
    slot = convert_slot(eq["slot"])
    tier = convert_tier(eq["tier"])
    cost = eq.get("cost", 0)

    sb = eq.get("statBonus", {})
    atk = sb.get("atk", 0)
    def_ = sb.get("def", 0)
    mag = sb.get("mag", 0)
    spd = sb.get("spd", 0)
    hp = sb.get("hp", 0)

    allowed = eq.get("allowedElements", [])
    allowed_str = ", ".join(ron_element(e) for e in allowed)

    ua = eq.get("unlocksAbility")
    unlocks = f'Some(AbilityId({ron_string(ua)}))' if ua else "None"

    sid = eq.get("setId")
    set_id = f'Some(SetId({ron_string(sid)}))' if sid else "None"

    # alwaysFirstTurn doesn't exist in JSON data, default false
    always_first_turn = ron_bool(eq.get("alwaysFirstTurn", False))

    # mana_bonus and hit_count_bonus don't exist in JSON, default None
    mana_bonus = "None"
    hit_count_bonus = "None"

    return f"""    (
        id: EquipmentId({ron_string(eid)}),
        name: {ron_string(name)},
        slot: {slot},
        tier: {tier},
        cost: {cost},
        allowed_elements: [{allowed_str}],
        stat_bonus: (atk: {atk}, def: {def_}, mag: {mag}, spd: {spd}, hp: {hp}),
        unlocks_ability: {unlocks},
        set_id: {set_id},
        always_first_turn: {always_first_turn},
        mana_bonus: {mana_bonus},
        hit_count_bonus: {hit_count_bonus},
    )"""


# ── Djinn Conversion ─────────────────────────────────────────────────

def convert_djinn(djinn_stat, djinn_abilities):
    """Convert djinn stat + abilities JSON to RON DjinnDef string."""
    did = djinn_stat["id"]
    name = djinn_stat["name"]
    element = ron_element(djinn_stat["element"])
    tier = int(djinn_stat["tier"])

    sb = djinn_stat.get("statBonuses", {})
    atk = sb.get("atk", 0)
    def_ = sb.get("def", 0)
    mag = sb.get("mag", 0)
    spd = sb.get("spd", 0)
    hp = sb.get("hp", 0)

    # Summon effect
    summon = djinn_stat.get("summon", {})
    base_damage = summon.get("baseDamage", 0)
    if base_damage > 0:
        summon_effect = f"""Some((
            damage: {base_damage},
            buff: None,
            status: None,
            heal: None,
        ))"""
    else:
        summon_effect = "None"

    # Ability pairs from djinn_abilities
    ab = djinn_abilities.get("abilities", {})

    same_set = ab.get("same", {}).get("set", [])
    same_standby = ab.get("same", {}).get("standby", [])
    counter_set = ab.get("counter", {}).get("set", [])
    counter_standby = ab.get("counter", {}).get("standby", [])
    neutral_set = ab.get("neutral", {}).get("set", [])
    neutral_standby = ab.get("neutral", {}).get("standby", [])

    def fmt_ability_list(lst):
        if not lst:
            return "[]"
        items = ", ".join(f'AbilityId({ron_string(a)})' for a in lst)
        return f"[{items}]"

    # same: set = good_abilities, standby = recovery_abilities
    # counter: set = good_abilities, standby = recovery_abilities
    # neutral: set = good_abilities, standby = recovery_abilities

    return f"""    (
        id: DjinnId({ron_string(did)}),
        name: {ron_string(name)},
        element: {element},
        tier: {tier},
        stat_bonus: (atk: {atk}, def: {def_}, mag: {mag}, spd: {spd}, hp: {hp}),
        summon_effect: {summon_effect},
        ability_pairs: (
            same: (
                good_abilities: {fmt_ability_list(same_set)},
                recovery_abilities: {fmt_ability_list(same_standby)},
            ),
            counter: (
                good_abilities: {fmt_ability_list(counter_set)},
                recovery_abilities: {fmt_ability_list(counter_standby)},
            ),
            neutral: (
                good_abilities: {fmt_ability_list(neutral_set)},
                recovery_abilities: {fmt_ability_list(neutral_standby)},
            ),
        ),
    )"""


# ── Encounter Conversion ─────────────────────────────────────────────

def convert_difficulty(d):
    if d is None:
        return "Easy"
    mapping = {
        "easy": "Easy",
        "medium": "Medium",
        "hard": "Hard",
        "boss": "Boss",
    }
    return mapping[d.lower()]


def convert_encounter(enc, enemy_counts):
    """Convert encounter JSON to RON EncounterDef string."""
    eid = enc["id"]
    name = enc["name"]
    difficulty = convert_difficulty(enc.get("difficulty"))

    # Count enemy occurrences
    enemies_raw = enc.get("enemies", [])
    enemy_count_map = {}
    for e in enemies_raw:
        enemy_count_map[e] = enemy_count_map.get(e, 0) + 1

    enemies_parts = []
    for enemy_id, count in enemy_count_map.items():
        enemies_parts.append(
            f"(enemy_id: EnemyId({ron_string(enemy_id)}), count: {count})"
        )
    enemies_str = ",\n            ".join(enemies_parts)

    reward = enc.get("reward", {})
    xp = reward.get("xp") or 0
    gold = reward.get("gold") or 0

    # Recruit unit
    recruit_unit = reward.get("unlockUnit")
    recruit = f'Some(UnitId({ron_string(recruit_unit)}))' if recruit_unit else "None"

    # Djinn reward
    djinn_reward_id = reward.get("djinn")
    djinn_reward = f'Some(DjinnId({ron_string(djinn_reward_id)}))' if djinn_reward_id else "None"

    # Equipment rewards
    equip = reward.get("equipment")
    equip_rewards = []
    if equip:
        if isinstance(equip, dict):
            etype = equip.get("type", "none")
            if etype == "fixed":
                item_id = equip.get("itemId")
                if item_id:
                    equip_rewards.append(item_id)
            elif etype == "choice":
                # Include all choices as possible rewards
                options = equip.get("options", [])
                equip_rewards.extend(options)
        elif isinstance(equip, str):
            equip_rewards.append(equip)

    if equip_rewards:
        equip_str = ", ".join(f'EquipmentId({ron_string(e)})' for e in equip_rewards)
        equip_ron = f"[{equip_str}]"
    else:
        equip_ron = "[]"

    return f"""    (
        id: EncounterId({ron_string(eid)}),
        name: {ron_string(name)},
        difficulty: {difficulty},
        enemies: [
            {enemies_str},
        ],
        xp_reward: {xp},
        gold_reward: {gold},
        recruit: {recruit},
        djinn_reward: {djinn_reward},
        equipment_rewards: {equip_ron},
    )"""


# ── Unit Conversion ──────────────────────────────────────────────────

# Unit base stats and growth rates (from game design, matching sample data format)
UNIT_STATS = {
    "adept": {
        "name": "Adept", "element": "Venus", "mana_contribution": 2,
        "base_stats": {"hp": 120, "atk": 14, "def": 12, "mag": 10, "spd": 10},
        "growth_rates": {"hp": 18, "atk": 3, "def": 3, "mag": 2, "spd": 2},
    },
    "war-mage": {
        "name": "War Mage", "element": "Mars", "mana_contribution": 2,
        "base_stats": {"hp": 105, "atk": 12, "def": 10, "mag": 16, "spd": 12},
        "growth_rates": {"hp": 16, "atk": 3, "def": 2, "mag": 4, "spd": 2},
    },
    "mystic": {
        "name": "Mystic", "element": "Mercury", "mana_contribution": 3,
        "base_stats": {"hp": 95, "atk": 10, "def": 8, "mag": 18, "spd": 11},
        "growth_rates": {"hp": 14, "atk": 2, "def": 2, "mag": 4, "spd": 2},
    },
    "ranger": {
        "name": "Ranger", "element": "Jupiter", "mana_contribution": 2,
        "base_stats": {"hp": 100, "atk": 13, "def": 9, "mag": 14, "spd": 14},
        "growth_rates": {"hp": 15, "atk": 3, "def": 2, "mag": 3, "spd": 3},
    },
    "sentinel": {
        "name": "Sentinel", "element": "Venus", "mana_contribution": 1,
        "base_stats": {"hp": 140, "atk": 12, "def": 18, "mag": 6, "spd": 8},
        "growth_rates": {"hp": 22, "atk": 2, "def": 4, "mag": 1, "spd": 1},
    },
    "stormcaller": {
        "name": "Stormcaller", "element": "Jupiter", "mana_contribution": 3,
        "base_stats": {"hp": 85, "atk": 8, "def": 7, "mag": 20, "spd": 13},
        "growth_rates": {"hp": 12, "atk": 2, "def": 1, "mag": 5, "spd": 3},
    },
    "blaze": {
        "name": "Blaze", "element": "Mars", "mana_contribution": 2,
        "base_stats": {"hp": 100, "atk": 16, "def": 10, "mag": 14, "spd": 12},
        "growth_rates": {"hp": 16, "atk": 4, "def": 2, "mag": 3, "spd": 2},
    },
    "karis": {
        "name": "Karis", "element": "Mercury", "mana_contribution": 3,
        "base_stats": {"hp": 90, "atk": 9, "def": 9, "mag": 17, "spd": 12},
        "growth_rates": {"hp": 13, "atk": 2, "def": 2, "mag": 4, "spd": 3},
    },
    "tyrell": {
        "name": "Tyrell", "element": "Mars", "mana_contribution": 1,
        "base_stats": {"hp": 110, "atk": 18, "def": 11, "mag": 8, "spd": 13},
        "growth_rates": {"hp": 17, "atk": 5, "def": 2, "mag": 1, "spd": 3},
    },
    "felix": {
        "name": "Felix", "element": "Venus", "mana_contribution": 2,
        "base_stats": {"hp": 130, "atk": 16, "def": 14, "mag": 8, "spd": 10},
        "growth_rates": {"hp": 20, "atk": 4, "def": 3, "mag": 2, "spd": 2},
    },
    "tower-champion": {
        "name": "Tower Champion", "element": "Venus", "mana_contribution": 2,
        "base_stats": {"hp": 150, "atk": 18, "def": 16, "mag": 12, "spd": 11},
        "growth_rates": {"hp": 22, "atk": 4, "def": 4, "mag": 3, "spd": 2},
    },
}


def convert_unit(unit_json):
    """Convert unit ability JSON + stats to RON UnitDef string."""
    uid = unit_json["unitId"]
    stats = UNIT_STATS.get(uid)
    if not stats:
        print(f"  WARNING: No stats for unit '{uid}', using defaults")
        stats = {
            "name": unit_json["unitName"], "element": unit_json["element"],
            "mana_contribution": 2,
            "base_stats": {"hp": 100, "atk": 12, "def": 10, "mag": 10, "spd": 10},
            "growth_rates": {"hp": 15, "atk": 3, "def": 2, "mag": 2, "spd": 2},
        }

    name = stats["name"]
    element = ron_element(stats["element"])
    mana = stats["mana_contribution"]
    bs = stats["base_stats"]
    gr = stats["growth_rates"]

    abilities_parts = []
    for ab in unit_json.get("abilities", []):
        abilities_parts.append(
            f'(level: {ab["unlockLevel"]}, ability_id: AbilityId({ron_string(ab["id"])}))'
        )
    abilities_str = ",\n            ".join(abilities_parts)

    return f"""    (
        id: UnitId({ron_string(uid)}),
        name: {ron_string(name)},
        element: {element},
        mana_contribution: {mana},
        base_stats: (hp: {bs["hp"]}, atk: {bs["atk"]}, def: {bs["def"]}, mag: {bs["mag"]}, spd: {bs["spd"]}),
        growth_rates: (hp: {gr["hp"]}, atk: {gr["atk"]}, def: {gr["def"]}, mag: {gr["mag"]}, spd: {gr["spd"]}),
        abilities: [
            {abilities_str},
        ],
    )"""


# ── Main ─────────────────────────────────────────────────────────────

def load_json(filename):
    path = SRC_DIR / filename
    with open(path, "r") as f:
        return json.load(f)


def write_ron(filename, entries):
    """Write a RON file as a list of entries."""
    path = OUT_DIR / filename
    content = "[\n" + ",\n".join(entries) + ",\n]\n"
    with open(path, "w") as f:
        f.write(content)
    return len(entries)


def main():
    os.makedirs(OUT_DIR, exist_ok=True)

    print("Converting v2 JSON -> RON...")
    print(f"  Source: {SRC_DIR}")
    print(f"  Output: {OUT_DIR}")
    print()

    # ── Abilities ──
    abilities_json = load_json("data_abilities_v3.json")

    # Collect all ability IDs from JSON
    ability_ids_in_json = {a["id"] for a in abilities_json}

    # Check equipment for referenced abilities that don't exist
    equipment_json_check = load_json("data_equipment.json")
    missing_abilities = set()
    for eq in equipment_json_check:
        ua = eq.get("unlocksAbility")
        if ua and ua not in ability_ids_in_json:
            missing_abilities.add(ua)

    # Also check djinn abilities
    djinn_abilities_check = load_json("data_djinn_abilities.json")
    for da in djinn_abilities_check:
        abilities = da.get("abilities", {})
        for compat_type in ("same", "counter", "neutral"):
            compat = abilities.get(compat_type, {})
            for ability_id in compat.get("set", []) + compat.get("standby", []):
                if ability_id not in ability_ids_in_json:
                    missing_abilities.add(ability_id)

    # Also check unit abilities
    units_json_check = load_json("data_unit_abilities.json")
    for u in units_json_check:
        for ab in u.get("abilities", []):
            if ab["id"] not in ability_ids_in_json:
                missing_abilities.add(ab["id"])

    # Generate stub abilities for missing ones
    STUB_ABILITIES = {
        # Equipment-unlocked weapon abilities (physical attacks)
        "shadow-step": {"name": "Shadow Step", "type": "physical", "element": None, "basePower": 28, "manaCost": 1, "targets": "single-enemy"},
        "radiant-stab": {"name": "Radiant Stab", "type": "physical", "element": None, "basePower": 35, "manaCost": 1, "targets": "single-enemy"},
        "storm-pierce": {"name": "Storm Pierce", "type": "physical", "element": "Jupiter", "basePower": 38, "manaCost": 2, "targets": "single-enemy"},
        "void-swipe": {"name": "Void Swipe", "type": "physical", "element": None, "basePower": 45, "manaCost": 2, "targets": "single-enemy"},
        "dawn-slash": {"name": "Dawn Slash", "type": "physical", "element": None, "basePower": 42, "manaCost": 2, "targets": "single-enemy"},
        "ether-shot": {"name": "Ether Shot", "type": "psynergy", "element": "Jupiter", "basePower": 35, "manaCost": 2, "targets": "single-enemy"},
        "seraphic-beam": {"name": "Seraphic Beam", "type": "psynergy", "element": "Mercury", "basePower": 50, "manaCost": 3, "targets": "single-enemy"},
        "halberd-sweep": {"name": "Halberd Sweep", "type": "physical", "element": None, "basePower": 30, "manaCost": 1, "targets": "single-enemy"},
        "javelin-throw": {"name": "Javelin Throw", "type": "physical", "element": "Jupiter", "basePower": 32, "manaCost": 1, "targets": "single-enemy"},
        "mystic-stab": {"name": "Mystic Stab", "type": "physical", "element": None, "basePower": 16, "manaCost": 0, "targets": "single-enemy"},
    }

    for ability_id in sorted(missing_abilities):
        stub = STUB_ABILITIES.get(ability_id)
        if stub:
            full = {
                "id": ability_id,
                "name": stub["name"],
                "type": stub["type"],
                "element": stub.get("element"),
                "manaCost": stub.get("manaCost", 0),
                "basePower": stub.get("basePower", 0),
                "targets": stub.get("targets", "single-enemy"),
                "unlockLevel": 1,
                "hitCount": None,
                "statusEffect": None,
                "buffEffect": None,
                "ignoreDefensePercent": None,
                "splashDamagePercent": None,
                "chainDamage": None,
                "shieldCharges": None,
                "damageReductionPercent": None,
                "healOverTime": None,
                "revive": None,
                "reviveHPPercent": None,
                "cleanse_type": None,
                "grantImmunity": None,
            }
        else:
            # Generic fallback stub for djinn abilities etc.
            full = {
                "id": ability_id,
                "name": ability_id.replace("-", " ").title(),
                "type": "psynergy",
                "element": None,
                "manaCost": 1,
                "basePower": 30,
                "targets": "single-enemy",
                "unlockLevel": 1,
                "hitCount": None,
                "statusEffect": None,
                "buffEffect": None,
                "ignoreDefensePercent": None,
                "splashDamagePercent": None,
                "chainDamage": None,
                "shieldCharges": None,
                "damageReductionPercent": None,
                "healOverTime": None,
                "revive": None,
                "reviveHPPercent": None,
                "cleanse_type": None,
                "grantImmunity": None,
            }
        abilities_json.append(full)
        ability_ids_in_json.add(ability_id)

    if missing_abilities:
        print(f"  (generated {len(missing_abilities)} stub abilities for missing references)")

    ability_entries = [convert_ability(a) for a in abilities_json]
    count = write_ron("abilities.ron", ability_entries)
    print(f"  abilities.ron: {count} entries")

    # ── Enemies ──
    enemies_json = load_json("data_enemies.json")
    enemy_entries = [convert_enemy(e, abilities_json) for e in enemies_json]
    count = write_ron("enemies.ron", enemy_entries)
    print(f"  enemies.ron:   {count} entries")

    # ── Equipment ──
    equipment_json = load_json("data_equipment.json")
    equip_entries = [convert_equipment(eq) for eq in equipment_json]
    count = write_ron("equipment.ron", equip_entries)
    print(f"  equipment.ron: {count} entries")

    # ── Djinn ──
    djinn_json = load_json("data_djinn.json")
    djinn_abilities_json = load_json("data_djinn_abilities.json")

    # Build lookup by djinnId
    djinn_ab_map = {da["djinnId"]: da for da in djinn_abilities_json}

    djinn_entries = []
    for d in djinn_json:
        ab = djinn_ab_map.get(d["id"], {"abilities": {}})
        djinn_entries.append(convert_djinn(d, ab))
    count = write_ron("djinn.ron", djinn_entries)
    print(f"  djinn.ron:     {count} entries")

    # ── Encounters ──
    encounters_json = load_json("data_encounters.json")
    encounter_entries = [convert_encounter(enc, {}) for enc in encounters_json]
    count = write_ron("encounters.ron", encounter_entries)
    print(f"  encounters.ron: {count} entries")

    # ── Units ──
    units_json = load_json("data_unit_abilities.json")
    unit_entries = [convert_unit(u) for u in units_json]
    count = write_ron("units.ron", unit_entries)
    print(f"  units.ron:     {count} entries")

    print()
    print("Done! Remember to copy config.ron: cp data/sample/config.ron data/full/config.ron")


if __name__ == "__main__":
    main()
