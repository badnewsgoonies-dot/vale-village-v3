//! Shared UI helpers for non-battle JRPG screens.

use bevy::color::Srgba;
use bevy::prelude::*;

use crate::shared::{DjinnId, EquipmentId, MapNodeId, NpcId, ShopId};

pub const BG_COLOR: Color = Color::Srgba(Srgba::new(10.0 / 255.0, 10.0 / 255.0, 26.0 / 255.0, 1.0));
pub const PANEL_BG: Color =
    Color::Srgba(Srgba::new(26.0 / 255.0, 26.0 / 255.0, 46.0 / 255.0, 0.95));
pub const BORDER_COLOR: Color =
    Color::Srgba(Srgba::new(204.0 / 255.0, 136.0 / 255.0, 68.0 / 255.0, 1.0));
pub const TEXT_COLOR: Color =
    Color::Srgba(Srgba::new(232.0 / 255.0, 213.0 / 255.0, 183.0 / 255.0, 1.0));
pub const TEXT_DIM: Color =
    Color::Srgba(Srgba::new(136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0, 1.0));
pub const HIGHLIGHT: Color =
    Color::Srgba(Srgba::new(1.0, 204.0 / 255.0, 102.0 / 255.0, 1.0));
pub const GOLD_COLOR: Color =
    Color::Srgba(Srgba::new(1.0, 215.0 / 255.0, 0.0, 1.0));

const BUTTON_BG: Color =
    Color::Srgba(Srgba::new(34.0 / 255.0, 34.0 / 255.0, 56.0 / 255.0, 1.0));
const BUTTON_PRESSED: Color =
    Color::Srgba(Srgba::new(140.0 / 255.0, 92.0 / 255.0, 44.0 / 255.0, 1.0));

#[derive(Component)]
pub struct ScreenEntity;

#[derive(Component, Clone)]
pub struct MenuButton {
    pub action: ButtonAction,
}

#[derive(Clone)]
pub enum ButtonAction {
    NewGame,
    Continue,
    TravelTo(MapNodeId),
    EnterShop(ShopId),
    TalkToNpc(NpcId),
    CollectDjinn(DjinnId),
    BuyItem(EquipmentId),
    LeaveToMap,
    LeaveToTown,
}

#[derive(Component, Clone, Copy)]
pub struct ButtonBaseColor(pub Color);

pub fn spawn_panel(
    parent: &mut ChildBuilder,
    node: Node,
    build_children: impl FnOnce(&mut ChildBuilder),
) {
    parent
        .spawn((
            Node {
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(16.0)),
                ..node
            },
            BackgroundColor(PANEL_BG),
            BorderColor(BORDER_COLOR),
        ))
        .with_children(build_children);
}

pub fn spawn_button(
    parent: &mut ChildBuilder,
    label: impl Into<String>,
    action: ButtonAction,
    font_size: f32,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(56.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(BUTTON_BG),
            BorderColor(BORDER_COLOR),
            BorderRadius::all(Val::Px(6.0)),
            MenuButton { action },
            ButtonBaseColor(BUTTON_BG),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label.into()),
                TextFont {
                    font_size,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

pub fn button_hover_system(
    mut interaction_q: Query<
        (&Interaction, &ButtonBaseColor, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuButton>),
    >,
) {
    for (interaction, base_color, mut background) in &mut interaction_q {
        background.0 = match interaction {
            Interaction::Pressed => BUTTON_PRESSED,
            Interaction::Hovered => HIGHLIGHT,
            Interaction::None => base_color.0,
        };
    }
}

pub fn despawn_screen(mut commands: Commands, screen_q: Query<Entity, With<ScreenEntity>>) {
    for entity in &screen_q {
        commands.entity(entity).despawn_recursive();
    }
}
