use bevy::prelude::*;

use super::{Item, Offer, caption, pay, terminal, ware};
use crate::game::{
    cpu::{Op, Socket},
    currency::Currencies,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, panel.spawn()).add_observer(plug);
}

/// Instruction shop on the right. Instructions are bought by dragging them onto a free ROM slot.
fn panel() -> impl Scene {
    let body = bsn_list! {
        @caption("INSTRUCTIONS")
        --
        @ware(Item::Rom((Op::Add, 1)), 5)
        --
        @ware(Item::Rom((Op::Mul, 2)), 20)
        --
        @caption("drag opcodes onto a CPU display")
    };
    bsn! { #InstructionShop @terminal("root@c0de:~/opcodes$ _", false, body) }
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
