#![allow(dead_code)]
//! Shop Domain — ShopState, buy/sell logic, stock management.
//!
//! Stock encoding in ShopState:
//!   None        → Unlimited (never removed)
//!   Some(count) → Limited with `count` remaining (count ≥ 1 by ItemCount invariant)
//!   missing key → Limited stock was depleted (removed on reaching 0)

use std::collections::HashMap;

use crate::shared::bounded_types::{Gold, ItemCount};
use crate::shared::{ItemId, ShopDef, ShopEvent, ShopId, ShopStock};

// ── ShopError ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ShopError {
    NotEnoughGold,
    OutOfStock,
    ItemNotInShop,
}

// ── ShopState ────────────────────────────────────────────────────────

/// Runtime stock remaining per shop.
#[derive(Debug, Clone, Default)]
pub struct ShopState {
    pub stock: HashMap<ShopId, HashMap<ItemId, Option<ItemCount>>>,
}

// ── init ─────────────────────────────────────────────────────────────

pub fn init_shop_state(defs: &[ShopDef]) -> ShopState {
    let mut stock = HashMap::new();
    for def in defs {
        let mut entry_map = HashMap::new();
        for entry in &def.inventory {
            let remaining = match entry.stock {
                ShopStock::Unlimited => None,
                ShopStock::Limited(count) => Some(count),
            };
            entry_map.insert(entry.item_id.clone(), remaining);
        }
        stock.insert(def.id, entry_map);
    }
    ShopState { stock }
}

// ── ───────────────────────────────────

pub fn can_buy(
    shop_state: &ShopState,
    shop_id: ShopId,
    item_id: &ItemId,
    player_gold: Gold,
    defs: &[ShopDef],
) -> Result<(), ShopError> {
    let entry = defs
        .iter()
        .find(|d| d.id == shop_id)
        .and_then(|d| d.inventory.iter().find(|e| &e.item_id == item_id))
        .ok_or(ShopError::ItemNotInShop)?;

    if player_gold.get() < entry.price.get() {
        return Err(ShopError::NotEnoughGold);
    }

    // Item in def but absent from state map means it was Limited and depleted.
    let stock_entry = shop_state
        .stock
        .get(&shop_id)
        .and_then(|m| m.get(item_id));

    match stock_entry {
        None => Err(ShopError::OutOfStock),     // depleted limited stock
        Some(None) => Ok(()),                   // unlimited
        Some(Some(_)) => Ok(()),                // limited, count ≥ 1 by ItemCount invariant
    }
}

// ── execute_buy ───────────────────────────────────────────────────────

/// Decrements limited stock (removes entry when depleted). Returns a Buy event.
pub fn execute_buy(
    shop_state: &mut ShopState,
    shop_id: ShopId,
    item_id: &ItemId,
) -> ShopEvent {
    if let Some(shop_map) = shop_state.stock.get_mut(&shop_id) {
        // Snapshot the current value before mutating to avoid borrow conflicts.
        let current = shop_map.get(item_id).copied();
        if let Some(Some(remaining)) = current {
            let new_count = remaining.get() - 1; // u8; safe since remaining ≥ 1
            if new_count == 0 {
                shop_map.remove(item_id);
            } else {
                shop_map.insert(item_id.clone(), Some(ItemCount::new(new_count)));
            }
        }
        // None (unlimited): no-op
    }
    ShopEvent::Buy {
        shop: shop_id,
        item: item_id.clone(),
        count: ItemCount::new(1),
    }
}

// ── ───────────────

/// Returns 50% of the buy price, rounded down.
pub fn sell_price(base_price: Gold) -> Gold {
    Gold::new(base_price.get() / 2)
}

// ── execute_sell ──────────────────────────────────────────────────────

pub fn execute_sell(item_id: &ItemId, count: ItemCount) -> ShopEvent {
    ShopEvent::Sell {
        item: item_id.clone(),
        count,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ShopEntry;

    fn make_def(id: u8, item: &str, price: u32, stock: ShopStock) -> ShopDef {
        ShopDef {
            id: ShopId(id),
            name: format!("Shop {id}"),
            inventory: vec![ShopEntry {
                item_id: ItemId(item.to_string()),
                price: Gold::new(price),
                stock,
            }],
        }
    }

    fn item(s: &str) -> ItemId {
        ItemId(s.to_string())
    }

    #[test]
    fn buy_success() {
        let defs = vec![make_def(1, "sword", 100, ShopStock::Unlimited)];
        let state = init_shop_state(&defs);
        assert_eq!(
            can_buy(&state, ShopId(1), &item("sword"), Gold::new(200), &defs),
            Ok(())
        );
    }

    #[test]
    fn buy_exact_gold() {
        let defs = vec![make_def(1, "ring", 50, ShopStock::Unlimited)];
        let state = init_shop_state(&defs);
        assert_eq!(
            can_buy(&state, ShopId(1), &item("ring"), Gold::new(50), &defs),
            Ok(())
        );
    }

    #[test]
    fn buy_insufficient_gold() {
        let defs = vec![make_def(1, "sword", 100, ShopStock::Unlimited)];
        let state = init_shop_state(&defs);
        assert_eq!(
            can_buy(&state, ShopId(1), &item("sword"), Gold::new(50), &defs),
            Err(ShopError::NotEnoughGold)
        );
    }

    #[test]
    fn buy_item_not_in_shop() {
        let defs = vec![make_def(1, "sword", 100, ShopStock::Unlimited)];
        let state = init_shop_state(&defs);
        assert_eq!(
            can_buy(&state, ShopId(1), &item("shield"), Gold::new(200), &defs),
            Err(ShopError::ItemNotInShop)
        );
    }

    #[test]
    fn limited_stock_depletion() {
        let defs = vec![make_def(1, "potion", 10, ShopStock::Limited(ItemCount::new(1)))];
        let mut state = init_shop_state(&defs);
        assert_eq!(
            can_buy(&state, ShopId(1), &item("potion"), Gold::new(100), &defs),
            Ok(())
        );
        execute_buy(&mut state, ShopId(1), &item("potion"));
        assert_eq!(
            can_buy(&state, ShopId(1), &item("potion"), Gold::new(100), &defs),
            Err(ShopError::OutOfStock)
        );
    }

    #[test]
    fn limited_stock_partial_depletion() {
        let defs = vec![make_def(1, "arrow", 5, ShopStock::Limited(ItemCount::new(3)))];
        let mut state = init_shop_state(&defs);
        execute_buy(&mut state, ShopId(1), &item("arrow"));
        execute_buy(&mut state, ShopId(1), &item("arrow"));
        // 1 remaining — still purchasable
        assert_eq!(
            can_buy(&state, ShopId(1), &item("arrow"), Gold::new(100), &defs),
            Ok(())
        );
        execute_buy(&mut state, ShopId(1), &item("arrow"));
        assert_eq!(
            can_buy(&state, ShopId(1), &item("arrow"), Gold::new(100), &defs),
            Err(ShopError::OutOfStock)
        );
    }

    #[test]
    fn unlimited_never_depletes() {
        let defs = vec![make_def(1, "herb", 5, ShopStock::Unlimited)];
        let mut state = init_shop_state(&defs);
        for _ in 0..10 {
            execute_buy(&mut state, ShopId(1), &item("herb"));
        }
        assert_eq!(
            can_buy(&state, ShopId(1), &item("herb"), Gold::new(100), &defs),
            Ok(())
        );
    }

    #[test]
    fn sell_price_is_half() {
        assert_eq!(sell_price(Gold::new(100)).get(), 50);
        assert_eq!(sell_price(Gold::new(101)).get(), 50); // rounds down
        assert_eq!(sell_price(Gold::new(1)).get(), 0);
        assert_eq!(sell_price(Gold::new(0)).get(), 0);
    }

    #[test]
    fn execute_sell_emits_event() {
        let id = item("herb");
        let count = ItemCount::new(3);
        match execute_sell(&id, count) {
            ShopEvent::Sell { item: i, count: c } => {
                assert_eq!(i.0, "herb");
                assert_eq!(c.get(), 3);
            }
            _ => panic!("expected Sell event"),
        }
    }

    #[test]
    fn execute_buy_emits_event() {
        let defs = vec![make_def(1, "sword", 100, ShopStock::Unlimited)];
        let mut state = init_shop_state(&defs);
        match execute_buy(&mut state, ShopId(1), &item("sword")) {
            ShopEvent::Buy { shop, item: i, count } => {
                assert_eq!(shop, ShopId(1));
                assert_eq!(i.0, "sword");
                assert_eq!(count.get(), 1);
            }
            _ => panic!("expected Buy event"),
        }
    }
}
