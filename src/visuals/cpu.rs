use bevy::{prelude::*, sprite::Anchor, text::LineHeight};

use super::{
    BEZEL, DIM, GREEN, ROW, RUST, SCREEN, currencies::usd_pop, hex, rectangle, shop::SHOP_WIDTH,
    sound, text,
};
use crate::{
    game::{
        board::BoardSlot,
        cpu::{Cpu, Instruction, Job, Opcode, ProgramFinished, ProgramStarted, Rom, RomEmpty},
    },
    interaction::Draggable,
};

const BODY: Color = hex("#2e2926");
const EDGE: Color = hex("#453d38");
const LID: Color = hex("#a6843c");
const SHINE: Color = hex("#c9a552");
const RIM: Color = hex("#6e5526");
const PIN: Color = hex("#9c9488");
const CABLE: Color = hex("#7d766e");
const WIRE: Color = hex("#b03020");
const DARK: Color = hex("#1c1816");
const PRINT: Color = hex("#8a8278");
const SCANLINE: Color = hex("#00000059");

/// Rust spots on the chip as `(position, size)`.
const RUST_SPOTS: [(Vec2, Vec2); 6] = [
    (vec2(150., 42.), vec2(18., 8.)),
    (vec2(163., 35.), vec2(8., 6.)),
    (vec2(-118., -50.), vec2(14., 6.)),
    (vec2(182., -40.), vec2(10., 10.)),
    (vec2(-60., 52.), vec2(6., 4.)),
    (vec2(96., -54.), vec2(12., 5.)),
];
/// How long a CPU with an empty ROM shakes, in seconds.
const SHAKE_SECONDS: f32 = 0.35;
/// Vertical distance between display rows.
const ROW_SPACING: f32 = 32.;
/// Board space for one CPU, big enough for the tallest one.
const CELL_SIZE: Vec2 = vec2(300., 440.);
/// Middle of the tallest CPU, measured from its position.
const CENTER: Vec2 = vec2(0., -155.);

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (show_program, show_progress, place_cpus));
}

/// Screen text showing the program, one line per ROM slot.
#[derive(Component, FromTemplate)]
struct ProgramText(Entity);

/// Filling part of the progress bar.
#[derive(Component, FromTemplate)]
struct ProgressBar(Entity);

/// Seconds until the CPU stops shaking.
#[derive(Component, Default, Clone)]
struct Shake(f32);

/// CPU chip with its model and clock, wired to a green screen showing its program.
pub fn cpu_visuals(cpu: Cpu) -> impl Scene {
    let screen_height = ROW_SPACING * cpu.rom_size as f32 + 28.;
    let program_y = (row_y(0) + row_y(cpu.rom_size - 1)) / 2.;
    let chip_label = format!("{}\n{}Hz", cpu.model, cpu.clock);
    let rom_label = format!("ROM {}", cpu.rom_size);
    let pins = (0..40).map(pin).collect::<Vec<_>>();
    let spots = RUST_SPOTS
        .into_iter()
        .map(|(position, size)| rectangle(RUST, position.extend(0.05), size))
        .collect::<Vec<_>>();
    let cable = (0..14)
        .map(|stripe| {
            let color = [WIRE, CABLE, PIN][if stripe == 0 { 0 } else { 1 + stripe % 2 }];
            rectangle(
                color,
                vec3(-52. + 8. * stripe as f32, -109., -0.2),
                vec2(8., 24.),
            )
        })
        .collect::<Vec<_>>();
    let screws = [
        vec2(-94., -125.),
        vec2(94., -125.),
        vec2(-94., -145. - screen_height),
        vec2(94., -145. - screen_height),
    ]
    .into_iter()
    .map(|position| rectangle(PIN, position.extend(0.1), Vec2::splat(5.)))
    .collect::<Vec<_>>();
    let numbers = (0..cpu.rom_size)
        .map(|slot| text(format!("{slot:02}"), DIM, 18., vec2(-80., row_y(slot))))
        .collect::<Vec<_>>();
    let scanlines = (0..(screen_height / 4.) as usize)
        .map(|line| {
            rectangle(
                SCANLINE,
                vec3(0., -131. - 4. * line as f32, 0.9),
                vec2(180., 1.5),
            )
        })
        .collect::<Vec<_>>();
    bsn! {
        Transform { scale: vec3(0., 0., 1.) }
        Visibility
        ProgramText(#ProgramText)
        ProgressBar(#ProgressFill)
        Shake
        on(squash)
        on(shake)
        on(pop_usd)
        Children [
            #Package
            @rectangle(BODY, vec3(0., -42., 0.), vec2(420., 130.))
            Transform { scale: Vec3::splat(0.6) }
            Pickable
            Children [
                @rectangle(EDGE, vec3(0., 62.5, 0.05), vec2(420., 5.))
                --
                @rectangle(DARK, vec3(0., -62.5, 0.05), vec2(420., 5.))
                --
                @rectangle(RIM, vec3(0., 0., 0.1), vec2(160., 90.))
                --
                @rectangle(LID, vec3(0., 0., 0.2), vec2(150., 80.))
                --
                @rectangle(SHINE, vec3(0., 37., 0.3), vec2(150., 6.))
                --
                @rectangle(DARK, vec3(-206., 0., 0.1), vec2(8., 30.))
                --
                @rectangle(DARK, vec3(-199., 0., 0.1), vec2(6., 22.))
                --
                @rectangle(DARK, vec3(-194., 0., 0.1), vec2(4., 12.))
                --
                @rectangle(DARK, vec3(-185., -45., 0.1), vec2(10., 10.))
                --
                @text(chip_label, PRINT, 22., vec2(-185., 0.))
                --
                {pins}
                --
                {spots}
            ]
            --
            #Display
            @rectangle(BEZEL, vec3(0., -135. - screen_height / 2., 0.), vec2(196., screen_height + 28.))
            Pickable
            --
            @rectangle(DARK, vec3(0., -129. - screen_height / 2., 0.1), vec2(184., screen_height + 4.))
            --
            @rectangle(SCREEN, vec3(0., -129. - screen_height / 2., 0.2), vec2(180., screen_height))
            --
            @text(rom_label, PRINT, 12., vec2(-84., -139. - screen_height))
            --
            @rectangle(GREEN, vec3(82., -139. - screen_height, 0.1), vec2(8., 5.))
            --
            @rectangle(ROW, vec3(0., -117. - screen_height, 0.3), vec2(168., 8.))
            --
            #ProgressFill
            @rectangle(GREEN, vec3(-84., -117. - screen_height, 0.4), vec2(168., 8.))
            Anchor::CENTER_LEFT
            Transform { scale: vec3(0., 1., 1.) }
            --
            #ProgramText
            @text("", GREEN, 18., vec2(-46., program_y))
            LineHeight::Px(ROW_SPACING)
            --
            {numbers}
            --
            {scanlines}
            --
            {cable}
            --
            {screws}
        ]
    }
}

/// Display row behind ROM slot `slot`.
pub fn rom_slot_visuals(slot: usize) -> impl Scene {
    bsn! {
        @rectangle(ROW, vec3(0., row_y(slot), 0.3), vec2(168., 28.))
    }
}

/// CPU with empty ROM rows, shown as the ghost before buying.
pub fn ghost_visuals(cpu: Cpu) -> impl Scene {
    let rows = (0..cpu.rom_size).map(rom_slot_visuals).collect::<Vec<_>>();
    bsn! { @cpu_visuals(cpu) Children [ {rows} ] }
}

/// Chip pin `index`, the first 20 on top and the rest below. Some are rusty.
fn pin(index: usize) -> impl Scene {
    let (x, side) = (
        -190. + 20. * (index % 20) as f32,
        if index < 20 { 1. } else { -1. },
    );
    let color = if rand::random_bool(0.15) { RUST } else { PIN };
    bsn! {
        @rectangle(color, vec3(x, 71. * side, -0.1), vec2(10., 12.))
        Children [ @rectangle(color, vec3(0., 13. * side, 0.), vec2(5., 14.)) ]
    }
}

fn row_y(slot: usize) -> f32 {
    -151. - ROW_SPACING * slot as f32
}

/// Instruction as written in ROM, like `ADD 1`.
impl std::fmt::Display for Instruction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Instruction(opcode, value) = self;
        let opcode = match opcode {
            Opcode::Add => "ADD",
            Opcode::Mul => "MUL",
        };
        write!(formatter, "{opcode} {value}")
    }
}

/// Squeezes the CPU and plays a click when its program starts.
fn squash(
    started: On<ProgramStarted>,
    mut commands: Commands,
    mut transforms: Query<&mut Transform>,
) -> Result {
    transforms.get_mut(started.cpu)?.scale *= vec3(0.9, 0.9, 1.);
    commands.spawn_scene(sound("audio/click.wav"));
    Ok(())
}

/// Shakes the CPU and buzzes when its ROM is empty.
fn shake(empty: On<RomEmpty>, mut commands: Commands, mut shakes: Query<&mut Shake>) -> Result {
    shakes.get_mut(empty.cpu)?.0 = SHAKE_SECONDS;
    commands.spawn_scene(sound("audio/error.wav"));
    Ok(())
}

/// Shows the earned money above the CPU.
fn pop_usd(finished: On<ProgramFinished>, mut commands: Commands) {
    let usd = finished.usd;
    if usd > 0 {
        commands
            .entity(finished.cpu)
            .queue_spawn_related_scenes::<Children>(bsn_list! { @usd_pop(usd) });
    }
}

/// Writes the program on the CPU screen, dashes for empty slots.
fn show_program(
    cpus: Query<(&Rom, &ProgramText), Changed<Rom>>,
    mut texts: Query<&mut Text2d>,
) -> Result {
    for (rom, program_text) in &cpus {
        let lines = rom
            .slots()
            .iter()
            .map(|slot| slot.map_or("--------".into(), |instruction| instruction.to_string()))
            .collect::<Vec<_>>();
        texts.get_mut(program_text.0)?.0 = lines.join("\n");
    }
    Ok(())
}

/// Fills the progress bar while the program runs.
fn show_progress(
    cpus: Query<(&ProgressBar, Option<&Job>)>,
    mut transforms: Query<&mut Transform>,
) -> Result {
    for (bar, job) in &cpus {
        transforms.get_mut(bar.0)?.scale.x = job.map_or(0., |job| job.0.min(1.));
    }
    Ok(())
}

/// Grid of CPUs between the shops, scaled down so they all fit.
pub struct Grid {
    columns: usize,
    rows: usize,
    scale: f32,
}

impl Grid {
    /// Grid for `count` CPUs, with the number of columns that shows them biggest.
    pub fn fit(count: usize, window: &Window) -> Self {
        let area = vec2(window.width() - 2. * SHOP_WIDTH, window.height());
        let fit = |columns: usize| {
            (area / CELL_SIZE / vec2(columns as f32, count.div_ceil(columns) as f32))
                .min_element()
                .min(1.)
        };
        let columns = (1..=count)
            .max_by(|&a, &b| fit(a).total_cmp(&fit(b)))
            .unwrap_or(1);
        Grid {
            columns,
            rows: count.div_ceil(columns),
            scale: fit(columns),
        }
    }

    /// Center of board `slot`.
    fn center(&self, slot: usize) -> Vec2 {
        let cell = vec2((slot % self.columns) as f32, (slot / self.columns) as f32);
        let middle = vec2((self.columns - 1) as f32, (self.rows - 1) as f32) / 2.;
        (cell - middle) * vec2(1., -1.) * CELL_SIZE * self.scale
    }

    /// Board slot nearest to `point`.
    pub fn nearest(&self, point: Vec2, count: usize) -> usize {
        (0..count)
            .min_by(|&a, &b| {
                self.center(a)
                    .distance(point)
                    .total_cmp(&self.center(b).distance(point))
            })
            .unwrap_or(0)
    }
}

/// Moves CPUs into their board slots, dragged ones follow the pointer.
fn place_cpus(
    time: Res<Time>,
    window: Single<&Window>,
    mut cpus: Query<(&BoardSlot, &mut Shake, &mut Transform, Option<&Draggable>)>,
) {
    let grid = Grid::fit(cpus.iter().count(), &window);
    for (slot, mut shake, mut transform, draggable) in &mut cpus {
        shake.0 = (shake.0 - time.delta_secs()).max(0.);
        if let Some(draggable) = draggable.filter(|draggable| draggable.dragging()) {
            transform.translation =
                (draggable.start + draggable.distance * vec2(1., -1.)).extend(0.);
        } else {
            let target = grid.center(slot.0) - CENTER * grid.scale
                + vec2((shake.0 * 60.).sin() * shake.0 * 120., 0.);
            transform
                .translation
                .smooth_nudge(&target.extend(0.), 12., time.delta_secs());
        }
        transform
            .scale
            .smooth_nudge(&vec3(grid.scale, grid.scale, 1.), 12., time.delta_secs());
    }
}
