//! Shop screen UI.

use bevy::prelude::*;

use crate::domains::shop;
use crate::shared::{
    bounded_types::Gold, EquipmentId, GameScreen, ItemId, ShopDef, ShopEntry, ShopId,
};
use crate::starter_data::starter_shop_defs;

use super::app_state::{AppState, CurrentShop, CurrentTown, GameStateRes, SaveDataRes};
use super::plugin::GameDataRes;
use super::ui_helpers::{
    despawn_screen, spawn_button, spawn_panel, ButtonAction, ButtonBaseColor, MenuButton,
    ScreenEntity, BG_COLOR, BORDER_COLOR, GOLD_COLOR, TEXT_COLOR, TEXT_DIM,
};

#[derive(Component)]
struct ShopInventoryColumn;

#[derive(Component)]
struct ShopInfoColumn;

#[derive(Resource)]
struct ShopStatusMessage(String);

pub struct ShopScreenPlugin;

impl Plugin for ShopScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Shop), setup_shop_screen)
            .add_systems(
                Update,
                (handle_shop_buttons, refresh_shop_screen)
                    .chain()
                    .run_if(in_state(AppState::Shop)),
            )
            .add_systems(OnExit(AppState::Shop), (despawn_screen, cleanup_shop_state));
    }
}

fn setup_shop_screen(
    mut commands: Commands,
    current_shop: Res<CurrentShop>,
    mut game_state: ResMut<GameStateRes>,
) {
    let shop_defs = starter_shop_defs();
    if game_state.0.shop_state.stock.is_empty() {
        game_state.0.shop_state = shop::init_shop_state(&shop_defs);
    }

    let shop_name = shop_defs
        .iter()
        .find(|shop_def| shop_def.id == current_shop.0)
        .map(|shop_def| shop_def.name.clone())
        .unwrap_or_else(|| "Shop".to_string());

    commands.insert_resource(ShopStatusMessage(format!("Browsing {}.", shop_name)));
    commands.spawn((Camera2d, ScreenEntity));
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BG_COLOR),
            ScreenEntity,
        ))
        .with_children(|root| {
            spawn_panel(
                root,
                Node {
                    width: Val::Px(980.0),
                    min_height: Val::Px(540.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(24.0),
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                |panel| {
                    panel.spawn((
                        Node {
                            flex_basis: Val::Percent(62.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        ShopInventoryColumn,
                    ));
                    panel.spawn((
                        Node {
                            flex_basis: Val::Percent(38.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(12.0),
                            ..default()
                        },
                        ShopInfoColumn,
                    ));
                },
            );
        });
}

fn refresh_shop_screen(
    mut commands: Commands,
    current_shop: Res<CurrentShop>,
    game_state: Res<GameStateRes>,
    save_data: Res<SaveDataRes>,
    game_data: Res<GameDataRes>,
    status: Res<ShopStatusMessage>,
    inventory_q: Query<Entity, With<ShopInventoryColumn>>,
    info_q: Query<Entity, With<ShopInfoColumn>>,
) {
    let shop_defs = starter_shop_defs();
    let Some(shop_def) = shop_defs
        .iter()
        .find(|shop_def| shop_def.id == current_shop.0)
    else {
        return;
    };

    let Ok(inventory_column) = inventory_q.get_single() else {
        return;
    };
    let Ok(info_column) = info_q.get_single() else {
        return;
    };

    commands.entity(inventory_column).despawn_descendants();
    commands.entity(info_column).despawn_descendants();

    let player_gold = Gold::new(save_data.0.gold);

    commands.entity(inventory_column).with_children(|column| {
        column.spawn((
            Text::new(shop_def.name.clone()),
            TextFont {
                font_size: 34.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        ));
        column.spawn((
            Text::new("Inventory"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(TEXT_DIM),
        ));

        for entry in &shop_def.inventory {
            let remaining = stock_remaining(&game_state.0.shop_state, shop_def.id, &entry.item_id);
            let enabled = shop::can_buy(
                &game_state.0.shop_state,
                shop_def.id,
                &entry.item_id,
                player_gold,
                &shop_defs,
            )
            .is_ok();
            spawn_shop_entry_button(
                column,
                entry,
                item_display_name(&game_data, &entry.item_id),
                remaining,
                enabled,
            );
        }
    });

    commands.entity(info_column).with_children(|column| {
        column.spawn((
            Text::new("Party Gold"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(TEXT_DIM),
        ));
        column.spawn((
            Text::new(format!("{}", player_gold.get())),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(GOLD_COLOR),
        ));
        column.spawn((
            Text::new(status.0.clone()),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(TEXT_DIM),
        ));
        spawn_button(column, "Leave Shop", ButtonAction::LeaveToTown, 24.0);
    });
}

fn handle_shop_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    current_shop: Res<CurrentShop>,
    mut game_state: ResMut<GameStateRes>,
    mut save_data: ResMut<SaveDataRes>,
    mut status: ResMut<ShopStatusMessage>,
    button_q: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
) {
    let shop_defs = starter_shop_defs();

    for (interaction, button) in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match &button.action {
            ButtonAction::BuyItem(item_equipment_id) => {
                let item_id = ItemId(item_equipment_id.0.clone());
                let player_gold = Gold::new(save_data.0.gold);
                match shop::can_buy(
                    &game_state.0.shop_state,
                    current_shop.0,
                    &item_id,
                    player_gold,
                    &shop_defs,
                ) {
                    Ok(()) => {
                        if let Some(entry) = find_shop_entry(&shop_defs, current_shop.0, &item_id) {
                            shop::execute_buy(
                                &mut game_state.0.shop_state,
                                current_shop.0,
                                &item_id,
                            );
                            let remaining_gold =
                                Gold::new(player_gold.get().saturating_sub(entry.price.get()));
                            save_data.0.gold = remaining_gold.get();
                            game_state.0.gold = remaining_gold;
                            status.0 = format!("Bought {}.", display_name_from_id(&item_id));
                        }
                    }
                    Err(shop::ShopError::NotEnoughGold) => {
                        status.0 = "Not enough gold.".to_string();
                    }
                    Err(shop::ShopError::OutOfStock) => {
                        status.0 = "That item is out of stock.".to_string();
                    }
                    Err(shop::ShopError::ItemNotInShop) => {
                        status.0 = "That item is not sold here.".to_string();
                    }
                }
            }
            ButtonAction::LeaveToTown => {
                if let GameScreen::Town(town_id) = game_state.0.screen {
                    commands.insert_resource(CurrentTown(town_id));
                }
                next_state.set(AppState::Town);
            }
            _ => {}
        }
    }
}

fn cleanup_shop_state(mut commands: Commands) {
    commands.remove_resource::<CurrentShop>();
    commands.remove_resource::<ShopStatusMessage>();
}

fn find_shop_entry<'a>(
    shop_defs: &'a [ShopDef],
    shop_id: ShopId,
    item_id: &ItemId,
) -> Option<&'a ShopEntry> {
    shop_defs
        .iter()
        .find(|shop_def| shop_def.id == shop_id)
        .and_then(|shop_def| {
            shop_def
                .inventory
                .iter()
                .find(|entry| &entry.item_id == item_id)
        })
}

fn stock_remaining(shop_state: &shop::ShopState, shop_id: ShopId, item_id: &ItemId) -> Option<u8> {
    match shop_state
        .stock
        .get(&shop_id)
        .and_then(|shop_items| shop_items.get(item_id))
    {
        Some(None) => None,
        Some(Some(count)) => Some(count.get()),
        None => Some(0),
    }
}

fn spawn_shop_entry_button(
    parent: &mut ChildBuilder,
    entry: &ShopEntry,
    item_name: String,
    remaining: Option<u8>,
    enabled: bool,
) {
    let stock_text = match remaining {
        None => "Stock: Unlimited".to_string(),
        Some(0) => "Stock: 0".to_string(),
        Some(count) => format!("Stock: {}", count),
    };
    let base_color = if enabled {
        Color::srgba(0.13, 0.13, 0.22, 1.0)
    } else {
        Color::srgba(0.07, 0.07, 0.12, 1.0)
    };
    let text_color = if enabled { TEXT_COLOR } else { TEXT_DIM };

    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(72.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(base_color),
            BorderColor(BORDER_COLOR),
            BorderRadius::all(Val::Px(6.0)),
            MenuButton {
                action: ButtonAction::BuyItem(EquipmentId(entry.item_id.0.clone())),
            },
            ButtonBaseColor(base_color),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(format!("{}  |  {} G", item_name, entry.price.get())),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(text_color),
            ));
            button.spawn((
                Text::new(stock_text),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(text_color),
            ));
        });
}

fn item_display_name(game_data: &GameDataRes, item_id: &ItemId) -> String {
    game_data
        .0
        .equipment
        .get(&EquipmentId(item_id.0.clone()))
        .map(|equipment| equipment.name.clone())
        .unwrap_or_else(|| display_name_from_id(item_id))
}

fn display_name_from_id(item_id: &ItemId) -> String {
    item_id
        .0
        .split('-')
        .map(capitalize_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize_word(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => {
            let mut out = first.to_uppercase().collect::<String>();
            out.push_str(chars.as_str());
            out
        }
        None => String::new(),
    }
}
