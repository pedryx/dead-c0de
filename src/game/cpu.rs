use bevy::{prelude::*, sprite::Anchor};
use rand::{Rng, seq::IndexedRandom};

use super::{
    currency::{Currencies, usd_pop},
    shop::SHOP_W,
};

const BODY: Color = Color::srgb_u8(0x2e, 0x29, 0x26);
const EDGE: Color = Color::srgb_u8(0x45, 0x3d, 0x38);
const LID: Color = Color::srgb_u8(0xa6, 0x84, 0x3c);
const SHINE: Color = Color::srgb_u8(0xc9, 0xa5, 0x52);
const RIM: Color = Color::srgb_u8(0x6e, 0x55, 0x26);
const PIN: Color = Color::srgb_u8(0x9c, 0x94, 0x88);
const CABLE: Color = Color::srgb_u8(0x7d, 0x76, 0x6e);
const WIRE: Color = Color::srgb_u8(0xb0, 0x30, 0x20);
const DARK: Color = Color::srgb_u8(0x1c, 0x18, 0x16);
const PRINT: Color = Color::srgb_u8(0x8a, 0x82, 0x78);
const SCAN: Color = Color::srgba(0., 0., 0., 0.35);
const GLARE: Color = Color::srgba(1., 1., 1., 0.03);
pub(super) const RUST: Color = Color::srgb_u8(0x8b, 0x3a, 0x1a);
pub(super) const AMBER: Color = Color::srgb_u8(0xff, 0xb0, 0x00);
pub(super) const BEZEL: Color = Color::srgb_u8(0x3b, 0x34, 0x30);
pub(super) const SCREEN: Color = Color::srgb_u8(0x06, 0x12, 0x08);
pub(super) const ROW: Color = Color::srgb_u8(0x0c, 0x22, 0x12);
pub(super) const GREEN: Color = Color::srgb_u8(0x33, 0xff, 0x66);
pub(super) const DIM: Color = Color::srgb_u8(0x1f, 0x7a, 0x3a);

/// Rust spots on the CPU body as `(x, y, w, h)`.
const SPOTS: [(f32, f32, f32, f32); 6] = [
    (150., 42., 18., 8.),
    (163., 35., 8., 6.),
    (-118., -50., 14., 6.),
    (182., -40., 10., 10.),
    (-60., 52., 6., 4.),
    (96., -54., 12., 5.),
];
const MODELS: [&str; 8] = ["DC4004", "DC8008", "DC8080", "DC6502", "DC6800", "DC8086", "DC1802", "DC2650"];

/// Grid cell taken by one CPU with the tallest display and rising pops.
const CELL: Vec2 = vec2(300., 440.);
/// Middle of the tallest CPU's drawing, relative to its origin.
const CENTER: Vec2 = vec2(0., -155.);
const BOOT: Instruction = (Op::Add, 1);
const BOOT_SPEC: Spec = Spec { model: "DC4004", rom: 4, clock: 1. };
const SHAKE_SECS: f32 = 0.35;
/// Pointer travel in pixels that turns a click into a drag.
const DRAG_SLOP: f32 = 8.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, (|| cpu(0, BOOT_SPEC)).spawn())
        .add_systems(Update, (work, arrange, show_slots));
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Op {
    Add,
    Mul,
}

/// Opcode and its operand.
pub(super) type Instruction = (Op, u64);

/// CPU model and stats.
#[derive(Clone, Copy)]
pub(super) struct Spec {
    pub(super) model: &'static str,
    /// ROM slots, the max program length.
    pub(super) rom: usize,
    /// Jobs per second.
    pub(super) clock: f32,
}

impl Spec {
    pub(super) fn random() -> Self {
        let mut rng = rand::rng();
        Spec {
            model: MODELS.choose(&mut rng).unwrap(),
            rom: rng.random_range(2..=6),
            clock: rng.random_range(2..=8) as f32 / 4.,
        }
    }
}

#[derive(Component, FromTemplate)]
pub(super) struct Cpu {
    /// Position in the CPU grid.
    index: usize,
    /// Jobs per second.
    clock: f32,
    /// Max instructions executed per job, bounds loops.
    steps: usize,
    /// Running job's completion, it pays out at 1.
    job: Option<f32>,
    /// Progress bar fill showing job progress.
    fill: Entity,
    /// Seconds left of the shake refusing a click without a program.
    shake: f32,
    /// Screen distance the player dragged the CPU while reordering.
    drag: Vec2,
    /// Grid position the CPU rests at.
    home: Vec2,
}

/// ROM slot of a CPU, shown as a display row. Jobs run the slots' instructions top to bottom.
#[derive(Component, Default, Clone)]
pub(super) struct Socket(pub(super) Option<Instruction>);

/// DIP-40 CPU printed with its model and clock, wired to a green phosphor display listing its ROM slots.
/// Only the first CPU boots with a program.
pub(super) fn cpu(index: usize, spec: Spec) -> impl Scene {
    let screen = 32. * spec.rom as f32 + 28.;
    let print = format!("{}\n{}Hz", spec.model, spec.clock);
    let label = format!("ROM {}", spec.rom);
    let pins = (0..40).map(|i| pin(i, (i * 5 + index * 3).is_multiple_of(7))).collect::<Vec<_>>();
    let spots = SPOTS.into_iter().map(|(x, y, w, h)| rect(RUST, w, h, x, y, 0.05)).collect::<Vec<_>>();
    let cable = (0..14)
        .map(|i| rect([WIRE, CABLE, PIN][if i == 0 { 0 } else { 1 + i % 2 }], 8., 24., -52. + 8. * i as f32, -109., -0.2))
        .collect::<Vec<_>>();
    let screws = [(-94., -125.), (94., -125.), (-94., -145. - screen), (94., -145. - screen)]
        .into_iter()
        .map(|(x, y)| rect(PIN, 5., 5., x, y, 0.1))
        .collect::<Vec<_>>();
    let numbers = (0..spec.rom).map(|i| text(format!("{i:02}"), DIM, 18., -80., row_y(i))).collect::<Vec<_>>();
    let sockets = (0..spec.rom).map(|i| socket(i, (index == 0 && i == 0).then_some(BOOT))).collect::<Vec<_>>();
    let scanlines = (0..(screen / 4.) as usize)
        .map(|i| rect(SCAN, 180., 1.5, 0., -131. - 4. * i as f32, 0.9))
        .collect::<Vec<_>>();
    bsn! {
        #Cpu
        Cpu { index, clock: {spec.clock}, steps: 8, fill: #ProgressFill }
        Transform { scale: {vec3(0., 0., 1.)} }
        Visibility
        on(click)
        on(hold)
        on(place)
        Children [
            #Chip
            @rect(BODY, 420., 130., 0., -42., 0.)
            Transform { scale: {Vec3::splat(0.6)} }
            Pickable
            Children [
                @rect(EDGE, 420., 5., 0., 62.5, 0.05)
                --
                @rect(DARK, 420., 5., 0., -62.5, 0.05)
                --
                @rect(RIM, 160., 90., 0., 0., 0.1)
                --
                @rect(LID, 150., 80., 0., 0., 0.2)
                --
                @rect(SHINE, 150., 6., 0., 37., 0.3)
                --
                @rect(DARK, 8., 30., -206., 0., 0.1)
                --
                @rect(DARK, 6., 22., -199., 0., 0.1)
                --
                @rect(DARK, 4., 12., -194., 0., 0.1)
                --
                @rect(DARK, 10., 10., -185., -45., 0.1)
                --
                @text(print, PRINT, 22., -185., 0.)
                --
                {pins}
                --
                {spots}
            ]
            --
            #Display
            @rect(BEZEL, 196., screen + 28., 0., -135. - screen / 2., 0.)
            Pickable
            --
            @rect(DARK, 184., screen + 4., 0., -129. - screen / 2., 0.1)
            --
            @rect(SCREEN, 180., screen, 0., -129. - screen / 2., 0.2)
            --
            @rect(GLARE, 70., 30., -45., -150., 0.95)
            --
            @text(label, PRINT, 12., -84., -139. - screen)
            --
            @rect(GREEN, 8., 5., 82., -139. - screen, 0.1)
            --
            #ProgressBar
            @rect(ROW, 168., 8., 0., -117. - screen, 0.3)
            --
            #ProgressFill
            @rect(GREEN, 168., 8., -84., -117. - screen, 0.4)
            Anchor::CENTER_LEFT
            Transform { scale: {vec3(0., 1., 1.)} }
            --
            {sockets}
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

/// DIP pin `i`, the first 20 on top, the rest below.
fn pin(i: usize, rusty: bool) -> impl Scene {
    let (x, side) = (-190. + 20. * (i % 20) as f32, if i < 20 { 1. } else { -1. });
    let (color, tip) = (if rusty { RUST } else { PIN }, 13. * side);
    bsn! {
        @rect(color, 10., 12., x, 71. * side, -0.1)
        Children [ @rect(color, 5., 14., 0., tip, 0.) ]
    }
}

/// Display row of ROM slot `slot`, a drop target for shop instructions.
fn socket(slot: usize, instruction: Option<Instruction>) -> impl Scene {
    bsn! {
        Socket(instruction)
        @rect(ROW, 168., 28., 0., row_y(slot), 0.3)
        Pickable
        Children [ @text("", GREEN, 18., -46., 0.) ]
    }
}

fn row_y(slot: usize) -> f32 {
    -151. - 32. * slot as f32
}

/// Solid `w`×`h` rectangle at `x`, `y`, `z`.
fn rect(color: Color, w: f32, h: f32, x: f32, y: f32, z: f32) -> impl Scene {
    bsn! {
        Sprite { color, custom_size: {Some(vec2(w, h))} }
        Transform { translation: {vec3(x, y, z)} }
    }
}

/// Left-aligned `size`px text at `x`, `y`.
fn text(text: impl Into<String>, color: Color, size: f32, x: f32, y: f32) -> impl Scene {
    let text = text.into();
    bsn! {
        Text2d(text)
        TextFont { font_size: FontSize::Px(size) }
        TextColor(color)
        Anchor::CENTER_LEFT
        Transform { translation: {vec3(x, y, 0.5)} }
    }
}

/// Instruction as written in a ROM slot, like `ADD 1`.
pub(super) fn mnemonic((op, value): Instruction) -> String {
    format!("{op:?} {value}").to_uppercase()
}

/// Writes each socket's instruction on its display row, dashes when empty.
fn show_slots(sockets: Query<(&Socket, &Children), Changed<Socket>>, mut texts: Query<&mut Text2d>) -> Result {
    for (socket, children) in &sockets {
        texts.get_mut(children[0])?.0 = socket.0.map_or("--------".into(), mnemonic);
    }
    Ok(())
}

/// Squashes the CPU, ticks and starts a job, or shakes and buzzes without a program.
/// Ignored while a job runs and at the end of a drag.
fn click(
    click: On<PointerClick>,
    mut commands: Commands,
    mut cpus: Query<(&mut Cpu, &mut Transform, &Children)>,
    sockets: Query<&Socket>,
) -> Result {
    let (mut cpu, mut transform, children) = cpus.get_mut(click.entity)?;
    if cpu.job.is_some() || cpu.drag.length() > DRAG_SLOP {
        return Ok(());
    }
    if !children.iter().any(|child| sockets.get(child).is_ok_and(|socket| socket.0.is_some())) {
        cpu.shake = SHAKE_SECS;
        commands.spawn_scene(bsn! { AudioPlayer("audio/error.wav") PlaybackSettings::DESPAWN });
        return Ok(());
    }
    transform.scale *= vec3(0.9, 0.9, 1.);
    cpu.job = Some(0.);
    commands.spawn_scene(bsn! { AudioPlayer("audio/click.wav") PlaybackSettings::DESPAWN });
    Ok(())
}

fn hold(drag: On<PointerDrag>, mut cpus: Query<&mut Cpu>) -> Result {
    cpus.get_mut(drag.entity)?.drag = drag.distance;
    Ok(())
}

/// Drops the held CPU into the nearest grid cell, swapping places with the CPU there.
fn place(drag: On<PointerDragEnd>, mut cpus: Query<(Entity, &mut Cpu)>) -> Result {
    let (_, held) = cpus.get(drag.entity)?;
    let at = held.home + vec2(held.drag.x, -held.drag.y);
    let nearest = cpus
        .iter()
        .min_by(|(_, a), (_, b)| a.home.distance(at).total_cmp(&b.home.distance(at)))
        .map(|(entity, _)| entity);
    if let Some(nearest) = nearest
        && let Ok([(_, mut held), (_, mut other)]) = cpus.get_many_mut([drag.entity, nearest])
    {
        std::mem::swap(&mut held.index, &mut other.index);
    }
    cpus.get_mut(drag.entity)?.1.drag = Vec2::ZERO;
    Ok(())
}

/// Advances jobs by clock speed and pays each finished job's program result.
fn work(
    time: Res<Time>,
    mut commands: Commands,
    mut currencies: ResMut<Currencies>,
    mut cpus: Query<(Entity, &mut Cpu, &Children)>,
    sockets: Query<&Socket>,
    mut transforms: Query<&mut Transform>,
) -> Result {
    for (entity, mut cpu, children) in &mut cpus {
        let Some(progress) = cpu.job.map(|job| job + time.delta_secs() * cpu.clock) else {
            continue;
        };
        cpu.job = (progress < 1.).then_some(progress);
        transforms.get_mut(cpu.fill)?.scale.x = cpu.job.unwrap_or(0.);
        if cpu.job.is_some() {
            continue;
        }
        let usd = children
            .iter()
            .filter_map(|child| sockets.get(child).ok()?.0)
            .take(cpu.steps)
            .fold(0, |acc, (op, value)| match op {
                Op::Add => acc + value,
                Op::Mul => acc * value,
            });
        if usd > 0 {
            currencies.usd += usd;
            commands
                .entity(entity)
                .queue_spawn_related_scenes::<Children>(bsn_list! { @usd_pop(usd) Transform { translation: {vec3(0., 30., 1.)} } });
        }
    }
    Ok(())
}

/// Fits all CPUs into a grid between the shops and eases them there, following drags and shaking refusals.
/// Keeps z and z scale fixed: Bevy 0.20 draws sprites in the z order they first rendered with.
fn arrange(time: Res<Time>, window: Single<&Window>, mut cpus: Query<(&mut Cpu, &mut Transform)>) {
    let n = cpus.iter().count();
    let area = vec2(window.width() - 2. * SHOP_W, window.height());
    let fit = |cols: usize| (area / CELL / vec2(cols as f32, n.div_ceil(cols) as f32)).min_element().min(1.);
    let cols = (1..=n).max_by(|&a, &b| fit(a).total_cmp(&fit(b))).unwrap_or(1);
    let (rows, scale) = (n.div_ceil(cols) as f32, fit(cols));
    for (mut cpu, mut transform) in &mut cpus {
        cpu.shake = (cpu.shake - time.delta_secs()).max(0.);
        let (col, row) = ((cpu.index % cols) as f32, (cpu.index / cols) as f32);
        cpu.home = (vec2(col - (cols - 1) as f32 / 2., (rows - 1.) / 2. - row) * CELL - CENTER) * scale;
        let shake = (cpu.shake * 60.).sin() * cpu.shake * 120.;
        let target = (cpu.home + vec2(shake + cpu.drag.x, -cpu.drag.y)).extend(0.);
        transform.translation.smooth_nudge(&target, 12., time.delta_secs());
        transform.scale.smooth_nudge(&vec3(scale, scale, 1.), 12., time.delta_secs());
    }
}
