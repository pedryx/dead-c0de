use bevy::prelude::*;

use super::{
    cpu::{Cpu, Instruction, Opcode, ProgramFinished, ProgramStarted, Rom},
    currencies::Currencies,
};
use crate::visuals::progression::{goal_visuals, progression_visuals, victory_visuals};

/// USD the player has to hold to win.
pub const GOAL_USD: u64 = 1_000;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, goal.spawn())
        .add_systems(Update, (unlock, win))
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
    /// Player owns this many CPUs.
    Cpus(usize),
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

/// Goal hint, shown once the player owns a second CPU.
fn goal() -> impl Scene {
    bsn! { #Goal Locked({vec![Unlock::Cpus(2)]}) @goal_visuals() }
}

/// Shows the victory once the player holds the goal USD.
fn win(
    time: Res<Time>,
    mut commands: Commands,
    currencies: Single<&Currencies>,
    mut won: Local<bool>,
) {
    if !*won && currencies.usd >= GOAL_USD {
        *won = true;
        let seconds = time.elapsed_secs();
        commands.spawn_scene(bsn! { #Victory @victory_visuals(seconds) });
    }
}

/// Unlocks mechanics gated behind usd.
fn unlock(
    mut commands: Commands,
    currencies: Single<&Currencies>,
    roms: Query<&Rom>,
    cpus: Query<(), With<Cpu>>,
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
            Unlock::Cpus(count) => cpus.iter().count() >= *count,
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
