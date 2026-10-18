use bevy::prelude::*;

use super::{
    cpu::{STARTING_CPU, STARTING_PROGRAM, cpu},
    progression::progression,
};
use crate::{
    interaction::Draggable,
    visuals::{board::board_visuals, cpu::Grid},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, board.spawn())
        .add_observer(reorder_cpus);
}

/// CPU slot on the board.
#[derive(Component, Clone, Default)]
pub struct BoardSlot(pub usize);

/// Area holding the CPUs.
#[derive(Component, Clone, Default)]
pub struct Board {
    /// Preview of the CPU being dragged over the board.
    pub ghost: Option<Entity>,
}

fn board() -> impl Scene {
    bsn! {
        #Board
        Board
        @board_visuals()
        Children [ @cpu(STARTING_CPU, 0, &STARTING_PROGRAM) Children [ @progression() ] ]
    }
}

/// Moves `entity` from board slot `from` to `to`.
pub fn make_room(
    slots: &mut Query<(Entity, &mut BoardSlot)>,
    entity: Option<Entity>,
    from: usize,
    to: usize,
) {
    for (other, mut slot) in slots.iter_mut() {
        if Some(other) == entity {
            slot.0 = to;
        } else if from < to && (from + 1..=to).contains(&slot.0) {
            slot.0 -= 1;
        } else if to < from && (to..from).contains(&slot.0) {
            slot.0 += 1;
        }
    }
}

/// Moves the dragged CPU to the board slot under the pointer.
fn reorder_cpus(
    over: On<PointerDragOver>,
    window: Single<&Window>,
    boards: Query<(), With<Board>>,
    draggables: Query<&Draggable>,
    parents: Query<&ChildOf>,
    mut slots: Query<(Entity, &mut BoardSlot)>,
) {
    let Some(cpu) = parents
        .iter_ancestors(over.dragged)
        .find(|&entity| draggables.get(entity).is_ok_and(Draggable::dragging))
    else {
        return;
    };
    let (true, Some(point), Ok((_, slot))) = (
        boards.contains(over.entity),
        over.hit.position,
        slots.get(cpu),
    ) else {
        return;
    };
    let (from, count) = (slot.0, slots.iter().count());
    make_room(
        &mut slots,
        Some(cpu),
        from,
        Grid::fit(count, &window).nearest(point.truncate(), count),
    );
}
