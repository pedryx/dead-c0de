use bevy::prelude::*;

use super::{
    board::{Board, BoardSlot, make_room},
    cpu::{Cpu, Instruction, Opcode, Rom, RomSlot, cpu},
    currencies::Currencies,
    progression::{Locked, Unlock},
};
use crate::{
    interaction::Draggable,
    visuals::{
        cpu::{Grid, ghost_visuals},
        shop::{hardware_shop_visuals, instruction_shop_visuals, shop_item_visuals},
    },
};

/// CPUs on sale at once.
const CPU_OFFERS: usize = 3;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, (instruction_shop.spawn(), hardware_shop.spawn()))
        .add_systems(Update, allow_dragging)
        .add_observer(decline_drag)
        .add_observer(buy_instruction)
        .add_observer(show_ghost)
        .add_observer(hide_ghost)
        .add_observer(buy_cpu);
}

/// What a shop card sells.
#[derive(Component, Clone, Copy)]
pub enum Offer {
    Instruction(Instruction),
    Cpu(Cpu),
    /// Refreshes the CPUs in hardware store.
    Reroll,
    /// Bought CPU, leaving its card empty until the next reroll.
    Sold,
}

/// Cost in USD, doubles with every purchase.
#[derive(Component, Clone, Default)]
pub struct Price(pub u64);

/// Card's `offer` was bought.
#[derive(EntityEvent)]
pub struct Purchased {
    #[event_target]
    pub card: Entity,
    pub offer: Offer,
}

/// Player tried to buy a card without enough money.
#[derive(EntityEvent)]
pub struct CannotAfford {
    #[event_target]
    pub card: Entity,
}

/// Shop selling instructions.
fn instruction_shop() -> impl Scene {
    let cards = bsn_list! {
        @shop_item(Offer::Instruction(Instruction(Opcode::Add, 1)), 5)
        --
        @shop_item(Offer::Instruction(Instruction(Opcode::Mul, 2)), 30)
        Locked({vec![Unlock::Usd(20), Unlock::Instructions(Opcode::Add, 3)]})
    };
    bsn! { #InstructionShop Locked({vec![Unlock::Usd(3)]}) @instruction_shop_visuals(cards) }
}

/// Shop selling random CPUs.
fn hardware_shop() -> impl Scene {
    let cards = (0..CPU_OFFERS)
        .map(|_| {
            let cpu = Cpu::random();
            shop_item(Offer::Cpu(cpu), cpu_price(cpu, 1))
        })
        .collect::<Vec<_>>();
    bsn! {
        #HardwareShop
        Locked({vec![Unlock::Usd(50), Unlock::FullRom]})
        @hardware_shop_visuals(cards, reroll_button())
    }
}

/// Card selling `offer` for `price` USD.
fn shop_item(offer: Offer, price: u64) -> impl Scene {
    bsn! { offer Price(price) @shop_item_visuals() }
}

/// Button rerolling the CPUs on sale.
fn reroll_button() -> impl Scene {
    let reroll = Offer::Reroll;
    bsn! { reroll Price(10) on(reroll_cpus) @shop_item_visuals() }
}

/// Price of `cpu`, higher for more ROM and clock, doubled for each owned CPU.
fn cpu_price(cpu: Cpu, owned: usize) -> u64 {
    ((25. * cpu.rom_size as f32 * cpu.clock) as u64) << (owned - 1)
}

/// Takes `price` from the player and doubles it, false when there is not enough money.
fn pay(currencies: &mut Currencies, price: &mut Price) -> bool {
    if currencies.usd < price.0 {
        return false;
    }
    currencies.usd -= price.0;
    price.0 *= 2;
    true
}

/// Makes only the cards the player can afford draggable.
fn allow_dragging(
    mut commands: Commands,
    currencies: Single<Ref<Currencies>>,
    offers: Query<(Entity, &Offer, Ref<Price>)>,
) {
    for (card, offer, price) in &offers {
        if let Offer::Instruction(_) | Offer::Cpu(_) = offer
            && (currencies.is_changed() || price.is_changed())
        {
            if currencies.usd >= price.0 {
                commands.entity(card).insert_if_new(Draggable::default());
            } else {
                commands.entity(card).remove::<Draggable>();
            }
        }
    }
}

/// Declines dragging a card the player can't afford.
fn decline_drag(
    drag: On<PointerDragStart>,
    mut commands: Commands,
    offers: Query<&Offer, Without<Draggable>>,
) {
    if let Ok(Offer::Instruction(_) | Offer::Cpu(_)) = offers.get(drag.entity) {
        commands.trigger(CannotAfford { card: drag.entity });
    }
}

/// Buys the instruction dropped on an empty ROM slot and writes it there.
fn buy_instruction(
    drop: On<PointerDragDrop>,
    mut commands: Commands,
    mut offers: Query<(&Offer, &mut Price), With<Draggable>>,
    slots: Query<(&RomSlot, &ChildOf)>,
    mut roms: Query<&mut Rom>,
    mut currencies: Single<&mut Currencies>,
) -> Result {
    let (Ok((&Offer::Instruction(instruction), mut price)), Ok((slot, child_of))) =
        (offers.get_mut(drop.dropped), slots.get(drop.entity))
    else {
        return Ok(());
    };
    let (card, mut rom) = (drop.dropped, roms.get_mut(child_of.parent())?);
    if !rom.is_free(slot.0) {
        return Ok(());
    }
    if pay(&mut currencies, &mut price) {
        rom.write(slot.0, instruction);
        commands.trigger(Purchased {
            card,
            offer: Offer::Instruction(instruction),
        });
    } else {
        commands.trigger(CannotAfford { card });
    }
    Ok(())
}

/// Shows a ghost of the dragged CPU in the board slot nearest the pointer.
fn show_ghost(
    over: On<PointerDragOver>,
    mut commands: Commands,
    window: Single<&Window>,
    mut boards: Query<&mut Board>,
    offers: Query<&Offer, With<Draggable>>,
    mut slots: Query<(Entity, &mut BoardSlot)>,
) {
    let (Ok(mut board), Ok(&Offer::Cpu(cpu)), Some(point)) = (
        boards.get_mut(over.entity),
        offers.get(over.dragged),
        over.hit.position,
    ) else {
        return;
    };
    let count = slots.iter().count() + usize::from(board.ghost.is_none());
    let slot = Grid::fit(count, &window).nearest(point.truncate(), count);
    match board.ghost {
        Some(ghost) => {
            if let Ok((_, from)) = slots.get(ghost) {
                let from = from.0;
                make_room(&mut slots, Some(ghost), from, slot);
            }
        }
        None => {
            let board_entity = over.entity;
            make_room(&mut slots, None, count - 1, slot);
            board.ghost = Some(
                commands
                    .spawn_scene(bsn! { BoardSlot(slot) ChildOf(board_entity) @ghost_visuals(cpu) })
                    .id(),
            );
        }
    }
}

/// Removes the ghost once the dragged CPU leaves the board.
fn hide_ghost(
    leave: On<PointerDragLeave>,
    mut commands: Commands,
    mut boards: Query<&mut Board>,
    mut slots: Query<(Entity, &mut BoardSlot)>,
) {
    if let Ok(mut board) = boards.get_mut(leave.entity)
        && let Some(ghost) = board.ghost.take()
    {
        remove_ghost(&mut commands, &mut slots, ghost);
    }
}

/// Buys the CPU dropped on the board and puts it where its ghost was.
fn buy_cpu(
    drop: On<PointerDragDrop>,
    mut commands: Commands,
    mut boards: Query<&mut Board>,
    mut offers: Query<(&mut Offer, &mut Price)>,
    mut slots: Query<(Entity, &mut BoardSlot)>,
    mut currencies: Single<&mut Currencies>,
) -> Result {
    let Ok(mut board) = boards.get_mut(drop.entity) else {
        return Ok(());
    };
    let Some(ghost) = board.ghost.take() else {
        return Ok(());
    };
    let (mut offer, mut price) = offers.get_mut(drop.dropped)?;
    let Offer::Cpu(bought) = *offer else {
        return Ok(());
    };
    let card = drop.dropped;
    if !pay(&mut currencies, &mut price) {
        remove_ghost(&mut commands, &mut slots, ghost);
        commands.trigger(CannotAfford { card });
        return Ok(());
    }
    *offer = Offer::Sold;
    commands.trigger(Purchased {
        card,
        offer: Offer::Cpu(bought),
    });
    for (offer, mut price) in &mut offers {
        if let Offer::Cpu(_) = *offer {
            price.0 *= 2;
        }
    }
    let (slot, board_entity) = (slots.get(ghost)?.1.0, drop.entity);
    commands.entity(ghost).despawn();
    commands
        .entity(board_entity)
        .queue_spawn_related_scenes::<Children>(bsn_list! { @cpu(bought, slot, &[]) });
    Ok(())
}

/// Removes `ghost` and closes the gap it left on the board.
fn remove_ghost(
    commands: &mut Commands,
    slots: &mut Query<(Entity, &mut BoardSlot)>,
    ghost: Entity,
) {
    if let Ok((_, slot)) = slots.get(ghost) {
        let (from, last) = (slot.0, slots.iter().count() - 1);
        make_room(slots, Some(ghost), from, last);
    }
    commands.entity(ghost).despawn();
}

/// Rerolls all CPUs in the hardware shop.
fn reroll_cpus(
    click: On<PointerClick>,
    mut commands: Commands,
    mut offers: Query<(&mut Offer, &mut Price)>,
    cpus: Query<(), With<Cpu>>,
    mut currencies: Single<&mut Currencies>,
) -> Result {
    let (card, (_, mut price)) = (click.entity, offers.get_mut(click.entity)?);
    if !pay(&mut currencies, &mut price) {
        commands.trigger(CannotAfford { card });
        return Ok(());
    }
    commands.trigger(Purchased {
        card,
        offer: Offer::Reroll,
    });
    let owned = cpus.iter().count();
    for (mut offer, mut price) in &mut offers {
        if let Offer::Cpu(_) | Offer::Sold = *offer {
            let cpu = Cpu::random();
            *offer = Offer::Cpu(cpu);
            price.0 = cpu_price(cpu, owned);
        }
    }
    Ok(())
}
