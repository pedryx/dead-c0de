use bevy::prelude::*;

use super::{
    cpu::{AMBER, BEZEL, Cpu, DIM, GREEN, Instruction, Op, ROW, RUST, SCREEN, Socket, cpu, mnemonic},
    currency::Currencies,
};

/// Screen width kept free for the shop on the right.
pub(super) const SHOP_W: f32 = 300.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, shop.spawn())
        .add_systems(Update, tint.run_if(resource_changed::<Currencies>))
        .add_observer(plug);
}

#[derive(Default, Clone)]
enum Item {
    #[default]
    Cpu,
    Rom(Instruction),
}

/// Buyable item, its price doubles with every purchase.
#[derive(Component, FromTemplate)]
struct Offer {
    item: Item,
    price: u64,
    /// Text showing the price.
    tag: Entity,
}

/// Terminal-style shop. Instructions are bought by dragging them onto a free ROM slot, CPUs by clicking.
fn shop() -> impl Scene {
    bsn! {
        #Shop
        Node {
            position_type: PositionType::Absolute,
            top: px(64.),
            right: px(16.),
            width: px(SHOP_W - 32.),
            flex_direction: FlexDirection::Column,
            border: {UiRect::all(px(4.))},
        }
        BackgroundColor(SCREEN)
        ~{BorderColor::all(BEZEL)}
        Children [
            Node { padding: {UiRect::axes(px(10.), px(6.))} }
            BackgroundColor(BEZEL)
            Children [ Text("root@c0de:~/shop$ _") TextFont { font_size: FontSize::Px(18.) } TextColor(AMBER) ]
            --
            Node { flex_direction: FlexDirection::Column, row_gap: px(8.), padding: {UiRect::all(px(12.))} }
            Children [
                @caption("INSTRUCTIONS")
                --
                @offer(Item::Rom((Op::Add, 1)), 5) on(lift) on(carry) on(release)
                --
                @offer(Item::Rom((Op::Mul, 2)), 20) on(lift) on(carry) on(release)
                --
                @caption("HARDWARE")
                --
                @offer(Item::Cpu, 100) on(buy)
                --
                @caption("drag opcodes onto a CPU display")
            ]
        ]
    }
}

fn caption(text: &'static str) -> impl Scene {
    bsn! { Text(text) TextFont { font_size: FontSize::Px(14.) } TextColor(DIM) }
}

fn offer(item: Item, price: u64) -> impl Scene {
    let name = match item {
        Item::Rom(instruction) => mnemonic(instruction),
        Item::Cpu => "CPU".into(),
    };
    let cost = format!("{price}$");
    bsn! {
        Offer { item, price, tag: #Price }
        Node {
            justify_content: JustifyContent::SpaceBetween,
            padding: {UiRect::all(px(10.))},
            border: {UiRect::all(px(2.))},
        }
        BackgroundColor(ROW)
        ~{BorderColor::all(DIM)}
        on(hover)
        on(unhover)
        Children [
            Text(name) TextFont { font_size: FontSize::Px(20.) } TextColor(GREEN) Pickable::IGNORE
            --
            #Price Text(cost) TextFont { font_size: FontSize::Px(20.) } TextColor(AMBER) Pickable::IGNORE
        ]
    }
}

/// Shows prices the player can't afford in rust.
fn tint(currencies: Res<Currencies>, offers: Query<&Offer>, mut colors: Query<&mut TextColor>) -> Result {
    for offer in &offers {
        colors.get_mut(offer.tag)?.0 = if currencies.usd < offer.price { RUST } else { AMBER };
    }
    Ok(())
}

fn hover(over: On<PointerOver>, mut borders: Query<&mut BorderColor>) -> Result {
    *borders.get_mut(over.entity)? = BorderColor::all(GREEN);
    Ok(())
}

fn unhover(out: On<PointerOut>, mut borders: Query<&mut BorderColor>) -> Result {
    *borders.get_mut(out.entity)? = BorderColor::all(DIM);
    Ok(())
}

/// Charges the offer's price and doubles it, false when the player can't afford it.
fn pay(currencies: &mut Currencies, offer: &mut Offer, texts: &mut Query<&mut Text>) -> Result<bool> {
    if currencies.usd < offer.price {
        return Ok(false);
    }
    currencies.usd -= offer.price;
    offer.price *= 2;
    texts.get_mut(offer.tag)?.0 = format!("{}$", offer.price);
    Ok(true)
}

fn buy(
    click: On<PointerClick>,
    mut commands: Commands,
    mut currencies: ResMut<Currencies>,
    mut offers: Query<&mut Offer>,
    mut texts: Query<&mut Text>,
    cpus: Query<&Cpu>,
) -> Result {
    if pay(&mut currencies, &mut *offers.get_mut(click.entity)?, &mut texts)? {
        commands.spawn_scene(cpu(cpus.iter().count()));
    }
    Ok(())
}

/// Lets hovers pass through the dragged offer to the ROM slots below it.
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

/// Buys the instruction dropped on an empty ROM slot and plugs it in.
fn plug(
    drop: On<PointerDragDrop>,
    mut currencies: ResMut<Currencies>,
    mut offers: Query<&mut Offer>,
    mut texts: Query<&mut Text>,
    mut sockets: Query<&mut Socket>,
) -> Result {
    let (Ok(mut offer), Ok(mut socket)) = (offers.get_mut(drop.dropped), sockets.get_mut(drop.entity)) else {
        return Ok(());
    };
    let Item::Rom(instruction) = offer.item else { return Ok(()) };
    if socket.0.is_none() && pay(&mut currencies, &mut offer, &mut texts)? {
        socket.0 = Some(instruction);
    }
    Ok(())
}
