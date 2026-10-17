use bevy::prelude::*;

use super::{Item, Offer, caption, hover, offer, pay, reprice, terminal, unhover, ware};
use crate::game::{
    cpu::{Cpu, Spec, cpu},
    currency::Currencies,
};

/// CPU cards on sale at once.
const DEALS: usize = 3;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, panel.spawn()).add_observer(deliver);
}

/// Price of `spec` while owning `owned` CPUs, doubling with every CPU bought.
fn price(spec: Spec, owned: usize) -> u64 {
    ((25. * spec.rom as f32 * spec.clock) as u64) << (owned - 1)
}

/// Hardware shop on the left with random CPUs, bought by dragging them onto the board.
fn panel() -> impl Scene {
    let deals = (0..DEALS)
        .map(|_| {
            let spec = Spec::random();
            ware(Item::Cpu(spec), price(spec, 1))
        })
        .collect::<Vec<_>>();
    let body = bsn_list! {
        @caption("CPUS")
        --
        {deals}
        --
        @offer(Item::Reroll, 10) on(hover) on(unhover) on(reroll)
        --
        @caption("drag a CPU onto the board")
    };
    bsn! { #HardwareShop @terminal("root@c0de:~/hw$ _", true, body) }
}

/// Buys the CPU card dropped outside the shops, leaving its slot empty until a reroll.
fn deliver(
    drop: On<PointerDragDrop>,
    mut commands: Commands,
    mut currencies: ResMut<Currencies>,
    windows: Query<(), With<Window>>,
    nodes: Query<(), With<Node>>,
    mut offers: Query<&mut Offer>,
    mut texts: Query<&mut Text>,
    cpus: Query<(), With<Cpu>>,
) -> Result {
    if !windows.contains(drop.entity) || nodes.contains(drop.original_event_target()) {
        return Ok(());
    }
    let Ok(offer) = offers.get(drop.dropped) else { return Ok(()) };
    let Item::Cpu(spec) = offer.item else { return Ok(()) };
    if currencies.usd < offer.price {
        return Ok(());
    }
    currencies.usd -= offer.price;
    commands.spawn_scene(cpu(cpus.iter().count(), spec));
    commands.entity(drop.dropped).insert(Visibility::Hidden);
    for mut offer in &mut offers {
        if matches!(offer.item, Item::Cpu(_)) {
            reprice(&mut offer, &mut texts)?;
        }
    }
    Ok(())
}

/// Pays for the reroll and restocks every CPU slot with a new random CPU.
fn reroll(
    click: On<PointerClick>,
    mut commands: Commands,
    mut currencies: ResMut<Currencies>,
    mut offers: Query<(Entity, &mut Offer)>,
    mut texts: Query<&mut Text>,
    cpus: Query<(), With<Cpu>>,
) -> Result {
    if !pay(&mut currencies, &mut offers.get_mut(click.entity)?.1, &mut texts)? {
        return Ok(());
    }
    for (entity, offer) in &offers {
        if matches!(offer.item, Item::Cpu(_)) {
            let spec = Spec::random();
            commands
                .entity(entity)
                .despawn_related::<Children>()
                .queue_apply_scene(super::offer(Item::Cpu(spec), price(spec, cpus.iter().count())));
        }
    }
    Ok(())
}
