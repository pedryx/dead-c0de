use bevy::prelude::*;
use rand::Rng;

use super::{board::BoardSlot, currencies::Currencies};
use crate::{
    interaction::Draggable,
    visuals::cpu::{cpu_visuals, rom_slot_visuals},
};

pub const STARTING_CPU: Cpu = Cpu {
    model: "DC4004",
    rom_size: 4,
    clock: 1.,
};
pub const STARTING_PROGRAM: [Instruction; 1] = [Instruction(Opcode::Add, 1)];
const MODEL_NAMES: [&str; 8] = [
    "DC4004", "DC8008", "DC8080", "DC6502", "DC6800", "DC8086", "DC1802", "DC2650",
];

pub fn plugin(app: &mut App) {
    app.add_systems(Update, advance_jobs);
}

#[derive(Clone, Copy, PartialEq)]
pub enum Opcode {
    Add,
    Mul,
}

/// Opcode and its value.
#[derive(Clone, Copy)]
pub struct Instruction(pub Opcode, pub u64);

/// CPU model and stats.
#[derive(Component, Clone, Copy)]
pub struct Cpu {
    /// Model name.
    pub model: &'static str,
    /// Max program length.
    pub rom_size: usize,
    /// Program runs per second.
    pub clock: f32,
}

impl Cpu {
    /// Create a random CPU.
    pub fn random() -> Self {
        let mut rng = rand::rng();
        Cpu {
            model: MODEL_NAMES[rng.random_range(0..MODEL_NAMES.len())],
            rom_size: rng.random_range(2..=6),
            clock: rng.random_range(2..=8) as f32 / 4.,
        }
    }
}

/// CPU program, one instruction per slot, run from top to bottom.
#[derive(Component, Clone)]
pub struct Rom(Vec<Option<Instruction>>);

impl Rom {
    /// Instruction in each slot, top to bottom.
    pub fn slots(&self) -> &[Option<Instruction>] {
        &self.0
    }

    /// Determine if `slot` is empty.
    pub fn is_free(&self, slot: usize) -> bool {
        self.0[slot].is_none()
    }

    /// Writes `instruction` into `slot`.
    pub fn write(&mut self, slot: usize, instruction: Instruction) {
        self.0[slot] = Some(instruction);
    }
}

/// ROM slot index, instructions are dropped on it.
#[derive(Component, Clone, Default)]
pub struct RomSlot(pub usize);

/// Progress of the running program, from 0 to 1.
#[derive(Component)]
pub struct Job(pub f32);

/// CPU started running its program.
#[derive(EntityEvent)]
pub struct ProgramStarted {
    #[event_target]
    pub cpu: Entity,
}

/// CPU was clicked with an empty ROM.
#[derive(EntityEvent)]
pub struct RomEmpty {
    #[event_target]
    pub cpu: Entity,
}

/// CPU finished its program, earning `usd`.
#[derive(EntityEvent)]
pub struct ProgramFinished {
    #[event_target]
    pub cpu: Entity,
    pub usd: u64,
}

/// CPU at `board_slot` with `program` in its first ROM slots.
pub fn cpu(cpu: Cpu, board_slot: usize, program: &[Instruction]) -> impl Scene {
    let rom = Rom((0..cpu.rom_size)
        .map(|slot| program.get(slot).copied())
        .collect());
    let slots = (0..cpu.rom_size)
        .map(|slot| bsn! { RomSlot(slot) Pickable @rom_slot_visuals(slot) })
        .collect::<Vec<_>>();
    bsn! {
        #Cpu
        cpu
        rom
        BoardSlot(board_slot)
        Draggable
        @cpu_visuals(cpu)
        on(run_program)
        Children [ {slots} ]
    }
}

/// Starts the program of the clicked CPU.
fn run_program(
    click: On<PointerClick>,
    mut commands: Commands,
    idle: Query<(&Rom, &Draggable), Without<Job>>,
) {
    let cpu = click.entity;
    let Ok((rom, draggable)) = idle.get(cpu) else {
        return;
    };
    if draggable.dragging() {
        return;
    }
    if rom.0.iter().any(Option::is_some) {
        commands.entity(cpu).insert(Job(0.));
        commands.trigger(ProgramStarted { cpu });
    } else {
        commands.trigger(RomEmpty { cpu });
    }
}

/// Advances jobs and pays out the finished ones.
fn advance_jobs(
    time: Res<Time>,
    mut commands: Commands,
    mut jobs: Query<(Entity, &Cpu, &Rom, &mut Job)>,
    mut currencies: Single<&mut Currencies>,
) {
    for (entity, cpu, rom, mut job) in &mut jobs {
        job.0 += time.delta_secs() * cpu.clock;
        if job.0 < 1. {
            continue;
        }
        let usd =
            rom.0
                .iter()
                .flatten()
                .fold(0, |total, Instruction(opcode, value)| match opcode {
                    Opcode::Add => total + value,
                    Opcode::Mul => total * value,
                });
        currencies.usd += usd;
        commands.entity(entity).remove::<Job>();
        commands.trigger(ProgramFinished { cpu: entity, usd });
    }
}
