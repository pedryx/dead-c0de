use bevy::prelude::*;

use super::{
    cpu::{AMBER, BEZEL, DIM, GREEN, Instruction, ROW, RUST, SCREEN, Spec, mnemonic},
    currency::Currencies,
};

mod hardware;
mod instructions;

/// Screen width taken by each shop.
pub(super) const SHOP_W: f32 = 300.;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((hardware::plugin, instructions::plugin))
        .add_systems(Update, tint.run_if(resource_changed::<Currencies>.or_eager(any_match_filter::<Changed<Offer>>)));
}

#[derive(Default, Clone, Copy)]
enum Item {
    #[default]
    Reroll,
    Rom(Instruction),
    Cpu(Spec),
}

/// Buyable item, its price doubles with every purchase.
#[derive(Component, FromTemplate)]
struct Offer {
    item: Item,
    price: u64,
    /// Text showing the price.
    tag: Entity,
}

/// Shop window styled as a terminal titled with a shell `prompt`, on the left or right screen edge.
fn terminal(prompt: &'static str, left: bool, body: impl SceneList) -> impl Scene {
    let (left, right) = if left { (px(16.), Val::Auto) } else { (Val::Auto, px(16.)) };
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(64.),
            left,
            right,
            width: px(SHOP_W - 32.),
            flex_direction: FlexDirection::Column,
            border: {UiRect::all(px(4.))},
        }
        BackgroundColor(SCREEN)
        ~{BorderColor::all(BEZEL)}
        Children [
            Node { padding: {UiRect::axes(px(10.), px(6.))} }
            BackgroundColor(BEZEL)
            Children [ Text(prompt) TextFont { font_size: FontSize::Px(16.) } TextColor(AMBER) ]
            --
            Node { flex_direction: FlexDirection::Column, row_gap: px(8.), padding: {UiRect::all(px(12.))} }
            Children [ {body} ]
        ]
    }
}

fn caption(text: &'static str) -> impl Scene {
    bsn! { Text(text) TextFont { font_size: FontSize::Px(14.) } TextColor(DIM) }
}

/// Card showing an item and its price.
fn offer(item: Item, price: u64) -> impl Scene {
    let (name, detail): (String, String) = match item {
        Item::Reroll => ("REROLL".into(), String::new()),
        Item::Rom(instruction) => (mnemonic(instruction), String::new()),
        Item::Cpu(spec) => (spec.model.into(), format!("\nROM {}  {}Hz", spec.rom, spec.clock)),
    };
    let cost = format!("{price}$");
    bsn! {
        Offer { item, price, tag: #Price }
        Node {
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: {UiRect::all(px(10.))},
            border: {UiRect::all(px(2.))},
        }
        Visibility::Inherited
        BackgroundColor(ROW)
        ~{BorderColor::all(DIM)}
        Children [
            Text(name) TextFont { font_size: FontSize::Px(20.) } TextColor(GREEN) Pickable::IGNORE
            Children [ TextSpan(detail) TextFont { font_size: FontSize::Px(14.) } TextColor(DIM) ]
            --
            #Price Text(cost) TextFont { font_size: FontSize::Px(20.) } TextColor(AMBER) Pickable::IGNORE
        ]
    }
}

/// Offer bought by dragging it out of the shop.
fn ware(item: Item, price: u64) -> impl Scene {
    bsn! { @offer(item, price) on(hover) on(unhover) on(lift) on(carry) on(release) }
}

/// Shows prices the player can't afford in rust.
fn tint(currencies: Res<Currencies>, offers: Query<&Offer>, mut colors: Query<&mut TextColor>) {
    for offer in &offers {
        if let Ok(mut color) = colors.get_mut(offer.tag) {
            color.0 = if currencies.usd < offer.price { RUST } else { AMBER };
        }
    }
}

fn hover(over: On<PointerOver>, mut borders: Query<&mut BorderColor>) -> Result {
    *borders.get_mut(over.entity)? = BorderColor::all(GREEN);
    Ok(())
}

fn unhover(out: On<PointerOut>, mut borders: Query<&mut BorderColor>) -> Result {
    *borders.get_mut(out.entity)? = BorderColor::all(DIM);
    Ok(())
}

/// Doubles the offer's price.
fn reprice(offer: &mut Offer, texts: &mut Query<&mut Text>) -> Result {
    offer.price *= 2;
    texts.get_mut(offer.tag)?.0 = format!("{}$", offer.price);
    Ok(())
}

/// Charges the offer's price and doubles it, false when the player can't afford it.
fn pay(currencies: &mut Currencies, offer: &mut Offer, texts: &mut Query<&mut Text>) -> Result<bool> {
    if currencies.usd < offer.price {
        return Ok(false);
    }
    currencies.usd -= offer.price;
    reprice(offer, texts)?;
    Ok(true)
}

/// Lets hovers pass through the dragged offer to whatever is below it.
fn lift(drag: On<PointerDragStart>, mut commands: Commands) {
    commands.entity(drag.entity).insert((Pickable::IGNORE, GlobalZIndex(1)));
}

fn carry(drag: On<PointerDrag>, mut transforms: Query<&mut UiTransform>) -> Result {
    transforms.get_mut(drag.entity)?.translation = Val2::px(drag.distance.x, drag.distance.y);
    Ok(())
}

/// Snaps the dragged offer back into the shop.
fn release(drag: On<PointerDragEnd>, mut commands: Commands) {
    commands.entity(drag.entity).insert((Pickable::default(), GlobalZIndex(0), UiTransform::default()));
}
