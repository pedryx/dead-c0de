use bevy::{prelude::*, sprite::Anchor};

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
const SCREWS: [(f32, f32); 4] = [(-94., -125.), (94., -125.), (-94., -301.), (94., -301.)];

/// Grid cell taken by one CPU with its display and rising pops.
const CELL: Vec2 = vec2(300., 400.);
/// Middle of the CPU's drawing, relative to its origin.
const CENTER: Vec2 = vec2(0., -122.);
/// ROM slots per CPU, the max program length.
const ROM: usize = 4;
const BOOT: Instruction = (Op::Add, 1);
const SHAKE_SECS: f32 = 0.35;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, (|| cpu(0)).spawn())
        .add_systems(Update, (work, arrange, show_slots));
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Op {
    Add,
    Mul,
}

/// Opcode and its operand.
pub(super) type Instruction = (Op, u64);

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
}

/// ROM slot of a CPU, shown as a display row. Jobs run the slots' instructions top to bottom.
#[derive(Component, Default, Clone)]
pub(super) struct Socket(pub(super) Option<Instruction>);

/// DIP-40 CPU wired to a green phosphor display listing its ROM slots. Only the first CPU boots with a program.
pub(super) fn cpu(index: usize) -> impl Scene {
    let pins = (0..40).map(|i| pin(i, (i * 5 + index * 3) % 7 == 0)).collect::<Vec<_>>();
    let spots = SPOTS.into_iter().map(|(x, y, w, h)| rect(RUST, w, h, x, y, 0.05)).collect::<Vec<_>>();
    let cable = (0..14)
        .map(|i| rect([WIRE, CABLE, PIN][if i == 0 { 0 } else { 1 + i % 2 }], 8., 24., -52. + 8. * i as f32, -109., -0.2))
        .collect::<Vec<_>>();
    let screws = SCREWS.into_iter().map(|(x, y)| rect(PIN, 5., 5., x, y, 0.1)).collect::<Vec<_>>();
    let numbers = (0..ROM).map(|i| text(format!("{i:02}"), DIM, 18., -80., row_y(i))).collect::<Vec<_>>();
    let sockets = (0..ROM).map(|i| socket(i, (index == 0 && i == 0).then_some(BOOT))).collect::<Vec<_>>();
    let scanlines = (0..39).map(|i| rect(SCAN, 180., 1.5, 0., -131. - 4. * i as f32, 0.9)).collect::<Vec<_>>();
    bsn! {
        #Cpu
        Cpu { index, clock: 1., steps: 8, fill: #ProgressFill }
        Transform { scale: {Vec3::ZERO} }
        Visibility
        on(click)
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
                @text("DC4004\n8213", PRINT, 16., -185., 0.)
                --
                {pins}
                --
                {spots}
            ]
            --
            #Display
            @rect(BEZEL, 196., 184., 0., -213., 0.)
            Pickable
            --
            @rect(DARK, 184., 160., 0., -207., 0.1)
            --
            @rect(SCREEN, 180., 156., 0., -207., 0.2)
            --
            @rect(GLARE, 70., 30., -45., -150., 0.95)
            --
            @text("DC-MON", PRINT, 10., -84., -295.)
            --
            @rect(GREEN, 8., 5., 82., -295., 0.1)
            --
            #ProgressBar
            @rect(ROW, 168., 8., 0., -273., 0.3)
            --
            #ProgressFill
            @rect(GREEN, 168., 8., -84., -273., 0.4)
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

/// Squashes the CPU, ticks and starts a job, or shakes and buzzes without a program. Ignored while a job runs.
fn click(
    click: On<PointerClick>,
    mut commands: Commands,
    mut cpus: Query<(&mut Cpu, &mut Transform, &Children)>,
    sockets: Query<&Socket>,
) -> Result {
    let (mut cpu, mut transform, children) = cpus.get_mut(click.entity)?;
    if cpu.job.is_some() {
        return Ok(());
    }
    if !children.iter().any(|child| sockets.get(child).is_ok_and(|socket| socket.0.is_some())) {
        cpu.shake = SHAKE_SECS;
        commands.spawn_scene(bsn! { AudioPlayer("audio/error.wav") PlaybackSettings::DESPAWN });
        return Ok(());
    }
    transform.scale *= 0.9;
    cpu.job = Some(0.);
    commands.spawn_scene(bsn! { AudioPlayer("audio/click.wav") PlaybackSettings::DESPAWN });
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
            commands.entity(entity).queue_spawn_related_scenes::<Children>(bsn_list! { @usd_pop(usd) Transform { translation: {vec3(0., 30., 1.)} } });
        }
    }
    Ok(())
}

/// Fits all CPUs into a grid left of the shop and eases them there, undoing click squashes and shaking refusals.
fn arrange(time: Res<Time>, window: Single<&Window>, mut cpus: Query<(&mut Cpu, &mut Transform)>) {
    let n = cpus.iter().count();
    let area = vec2(window.width() - SHOP_W, window.height());
    let fit = |cols: usize| (area / CELL / vec2(cols as f32, n.div_ceil(cols) as f32)).min_element().min(1.);
    let cols = (1..=n).max_by(|&a, &b| fit(a).total_cmp(&fit(b))).unwrap_or(1);
    let (rows, scale) = (n.div_ceil(cols) as f32, fit(cols));
    for (mut cpu, mut transform) in &mut cpus {
        cpu.shake = (cpu.shake - time.delta_secs()).max(0.);
        let (col, row) = ((cpu.index % cols) as f32, (cpu.index / cols) as f32);
        let cell = vec2(col - (cols - 1) as f32 / 2., (rows - 1.) / 2. - row);
        let shake = vec2((cpu.shake * 60.).sin() * cpu.shake * 120., 0.);
        let target = ((cell * CELL - CENTER) * scale - vec2(SHOP_W / 2., 0.) + shake).extend(0.);
        transform.translation.smooth_nudge(&target, 12., time.delta_secs());
        transform.scale.smooth_nudge(&Vec3::splat(scale), 12., time.delta_secs());
    }
}
