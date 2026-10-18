use bevy::prelude::*;

use super::{AMBER, BEZEL, DIM, GREEN, ROW, RUST, SCREEN, hex, sound, sound_with_volume};
use crate::{
    game::{
        cpu::{Rom, RomSlot},
        currencies::Currencies,
        shop::{CannotAfford, Offer, Price, Purchased},
    },
    interaction::Draggable,
};

const SLOT_HIGHLIGHT: Color = hex("#17502a");
/// Screen width taken by each shop.
pub const SHOP_WIDTH: f32 = 300.;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (label_cards, show_prices, drag_cards))
        .add_observer(highlight_slot)
        .add_observer(unhighlight_slot)
        .add_observer(play_lift_sound)
        .add_observer(play_drop_sound)
        .add_observer(play_buy_sound)
        .add_observer(play_cannot_afford_sound);
}

/// Texts of a shop card.
#[derive(Component, FromTemplate)]
struct ShopItemText {
    name: Entity,
    /// Stats under the name.
    detail: Entity,
    price: Entity,
}

/// Instruction shop window on the right.
pub fn instruction_shop_visuals(cards: impl SceneList) -> impl Scene {
    let body = bsn_list! {
        @caption("INSTRUCTIONS")
        --
        {cards}
        --
        @caption("drag onto a free ROM slot to buy it")
    };
    bsn! { @terminal("root@c0de:~/opcodes$ _", false, body) }
}

/// CPU shop window on the left.
pub fn hardware_shop_visuals(cards: impl SceneList, reroll: impl Scene) -> impl Scene {
    let body = bsn_list! {
        @caption("CPUS")
        --
        {cards}
        --
        @reroll
        --
        @caption("drag a CPU onto the board to buy it")
    };
    bsn! { @terminal("root@c0de:~/hw$ _", true, body) }
}

/// Shop card with a name and a price.
pub fn shop_item_visuals() -> impl Scene {
    bsn! {
        ShopItemText { name: #Name, detail: #Detail, price: #Price }
        Node {
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(10.)),
            border: UiRect::all(px(2.)),
        }
        BackgroundColor(ROW)
        BorderColor::all(DIM)
        GlobalZIndex(1)
        on(highlight_card)
        on(unhighlight_card)
        Children [
            #Name
            Text
            TextFont { font_size: FontSize::Px(20.) }
            TextColor(GREEN)
            Pickable::IGNORE
            Children [ #Detail TextSpan TextFont { font_size: FontSize::Px(14.) } TextColor(DIM) ]
            --
            #Price Text TextFont { font_size: FontSize::Px(20.) } Pickable::IGNORE
        ]
    }
}

/// Shop window titled `prompt`, on the left or right.
fn terminal(prompt: &'static str, left: bool, body: impl SceneList) -> impl Scene {
    let (left, right) = if left {
        (px(16.), Val::Auto)
    } else {
        (Val::Auto, px(16.))
    };
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(64.),
            left,
            right,
            width: px(SHOP_WIDTH - 32.),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(4.)),
        }
        BackgroundColor(SCREEN)
        BorderColor::all(BEZEL)
        Children [
            Node { padding: UiRect::axes(px(10.), px(6.)) }
            BackgroundColor(BEZEL)
            Children [ Text(prompt) TextFont { font_size: FontSize::Px(16.) } TextColor(AMBER) ]
            --
            Node { flex_direction: FlexDirection::Column, row_gap: px(8.), padding: UiRect::all(px(12.)) }
            Children [ {body} ]
        ]
    }
}

fn caption(text: &'static str) -> impl Scene {
    bsn! { Text(text) TextFont { font_size: FontSize::Px(14.) } TextColor(DIM) }
}

/// Updates card names and hides sold cards.
fn label_cards(
    mut cards: Query<(&Offer, &ShopItemText, &mut Visibility), Changed<Offer>>,
    mut names: Query<&mut Text>,
    mut details: Query<&mut TextSpan>,
) -> Result {
    for (offer, card, mut visibility) in &mut cards {
        *visibility = if let Offer::Sold = offer {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        let (name, detail) = match offer {
            Offer::Instruction(instruction) => (instruction.to_string(), String::new()),
            Offer::Cpu(cpu) => (
                cpu.model.into(),
                format!("\nROM {}  {}Hz", cpu.rom_size, cpu.clock),
            ),
            Offer::Reroll => ("REROLL".into(), String::new()),
            Offer::Sold => continue,
        };
        names.get_mut(card.name)?.0 = name;
        details.get_mut(card.detail)?.0 = detail;
    }
    Ok(())
}

/// Updates card prices, red when the player can't afford them.
fn show_prices(
    currencies: Single<Ref<Currencies>>,
    cards: Query<(Ref<Price>, &ShopItemText)>,
    mut texts: Query<(&mut Text, &mut TextColor)>,
) -> Result {
    for (price, card) in &cards {
        if currencies.is_changed() || price.is_changed() {
            let (mut text, mut color) = texts.get_mut(card.price)?;
            text.0 = format!("{}$", price.0);
            color.0 = if currencies.usd < price.0 {
                RUST
            } else {
                AMBER
            };
        }
    }
    Ok(())
}

/// Moves dragged cards with the pointer.
fn drag_cards(mut cards: Query<(&Draggable, &mut UiTransform), Changed<Draggable>>) {
    for (draggable, mut transform) in &mut cards {
        transform.translation = Val2::px(draggable.distance.x, draggable.distance.y);
    }
}

fn highlight_card(
    over: On<PointerOver>,
    mut commands: Commands,
    mut borders: Query<&mut BorderColor>,
) -> Result {
    *borders.get_mut(over.entity)? = BorderColor::all(GREEN);
    commands.spawn_scene(sound_with_volume("audio/hover.wav", 0.5));
    Ok(())
}

fn unhighlight_card(out: On<PointerOut>, mut borders: Query<&mut BorderColor>) -> Result {
    *borders.get_mut(out.entity)? = BorderColor::all(DIM);
    Ok(())
}

/// Highlights an empty ROM slot when an instruction is dragged over it.
fn highlight_slot(
    enter: On<PointerDragEnter>,
    mut commands: Commands,
    offers: Query<&Offer, With<Draggable>>,
    slots: Query<(&RomSlot, &ChildOf)>,
    roms: Query<&Rom>,
    mut sprites: Query<&mut Sprite>,
) -> Result {
    let Ok((slot, child_of)) = slots.get(enter.entity) else {
        return Ok(());
    };
    if let Ok(Offer::Instruction(_)) = offers.get(enter.dragged)
        && roms.get(child_of.parent())?.is_free(slot.0)
    {
        sprites.get_mut(enter.entity)?.color = SLOT_HIGHLIGHT;
        commands.spawn_scene(sound("audio/target.wav"));
    }
    Ok(())
}

fn unhighlight_slot(leave: On<PointerDragLeave>, mut slots: Query<&mut Sprite, With<RomSlot>>) {
    if let Ok(mut sprite) = slots.get_mut(leave.entity) {
        sprite.color = ROW;
    }
}

fn play_lift_sound(
    drag: On<PointerDragStart>,
    mut commands: Commands,
    offers: Query<(), With<Offer>>,
) {
    if offers.contains(drag.entity) {
        commands.spawn_scene(sound("audio/lift.wav"));
    }
}

fn play_drop_sound(
    drag: On<PointerDragEnd>,
    mut commands: Commands,
    offers: Query<(), With<Offer>>,
) {
    if offers.contains(drag.entity) {
        commands.spawn_scene(sound("audio/drop.wav"));
    }
}

/// Plays a sound for what was bought.
fn play_buy_sound(purchased: On<Purchased>, mut commands: Commands) {
    commands.spawn_scene(match purchased.offer {
        Offer::Instruction(_) => sound_with_volume("audio/program.wav", 0.7),
        Offer::Cpu(_) => sound_with_volume("audio/install.wav", 0.4),
        Offer::Reroll | Offer::Sold => sound_with_volume("audio/reroll.wav", 0.4),
    });
}

fn play_cannot_afford_sound(_: On<CannotAfford>, mut commands: Commands) {
    commands.spawn_scene(sound("audio/error.wav"));
}
