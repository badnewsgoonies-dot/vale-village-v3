# Domain: shop
## Scope: src/domains/shop/
## Imports from contract
ShopDef, ShopEntry, ShopStock, ShopEvent, ShopId, ItemId, Gold, ItemCount

## Deliverables
1. ShopState struct: remaining stock per shop (HashMap<ShopId, HashMap<ItemId, Option<ItemCount>>>)
2. fn init_shop_state(defs: &[ShopDef]) -> ShopState
3. fn can_buy(shop_state, shop_id, item_id, player_gold: Gold) -> Result<(), ShopError>
4. fn execute_buy(shop_state, shop_id, item_id) -> ShopEvent — decrements stock if Limited
5. fn sell_price(item_id, base_price: Gold) -> Gold — 50% of buy price
6. fn execute_sell(item_id, count: ItemCount) -> ShopEvent
7. ShopError enum: NotEnoughGold, OutOfStock, ItemNotInShop
8. Tests: buy success, buy insufficient gold, limited stock depletion, unlimited never depletes, sell pricing

## Does NOT handle
Inventory management (returns events), rendering shop UI, gold transactions (caller applies).

## Quantitative targets
- 2 ShopStock variants handled
- 2 ShopEvent variants emitted
- ≥8 tests
