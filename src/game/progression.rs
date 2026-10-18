use bevy::prelude::*;

use super::{
    cpu::{Instruction, Opcode, ProgramFinished, ProgramStarted, Rom},
    currencies::Currencies,
};
use crate::visuals::progression::progression_visuals;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, unlock)
        .add_observer(start_first_run)
        .add_observer(finish_run);
}

/// Hidden until any of the unlocks is met.
#[derive(Component, Clone, Default)]
pub struct Locked(pub Vec<Unlock>);

/// Way to unlock something.
#[derive(Clone)]
pub enum Unlock {
    /// Player owns this much USD.
    Usd(u64),
    /// A CPU's ROM is full.
    FullRom,
    /// CPUs hold this many instructions with the opcode.
    Instructions(Opcode, usize),
}

/// How far the player got.
#[derive(Component, Clone, Copy, Default, PartialEq)]
pub enum Progression {
    /// No program has run yet.
    #[default]
    Start,
    /// First program is running.
    FirstRun,
    /// First program paid out.
    FirstPay,
    /// Second program paid out.
    SecondPay,
}

/// Player progress, hinted above the starting CPU.
pub fn progression() -> impl Scene {
    bsn! { #Progression Progression @progression_visuals() }
}

/// Unlocks mechanics gated behind usd.
fn unlock(
    mut commands: Commands,
    currencies: Single<&Currencies>,
    roms: Query<&Rom>,
    locked: Query<(Entity, &Locked)>,
) {
    for (entity, locked) in &locked {
        let met = locked.0.iter().any(|unlock| match unlock {
            Unlock::Usd(usd) => currencies.usd >= *usd,
            Unlock::FullRom => roms
                .iter()
                .any(|rom| rom.slots().iter().all(Option::is_some)),
            Unlock::Instructions(opcode, count) => {
                roms.iter()
                    .flat_map(|rom| rom.slots().iter().flatten())
                    .filter(|Instruction(held, _)| held == opcode)
                    .count()
                    >= *count
            }
        });
        if met {
            commands.entity(entity).remove::<Locked>();
        }
    }
}

fn start_first_run(_: On<ProgramStarted>, mut progression: Single<&mut Progression>) {
    if **progression == Progression::Start {
        **progression = Progression::FirstRun;
    }
}

fn finish_run(_: On<ProgramFinished>, mut progression: Single<&mut Progression>) {
    **progression = match **progression {
        Progression::FirstRun => Progression::FirstPay,
        Progression::FirstPay => Progression::SecondPay,
        other => other,
    };
}
