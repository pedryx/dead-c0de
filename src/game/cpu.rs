use bevy::{prelude::*, sprite::Anchor};

use super::currency::{usd_pop, Currencies};

const BODY: Color = Color::srgb_u8(0x2e, 0x29, 0x26);
const LID: Color = Color::srgb_u8(0xa6, 0x84, 0x3c);
const PIN: Color = Color::srgb_u8(0x9c, 0x94, 0x88);
const DARK: Color = Color::srgb_u8(0x1c, 0x18, 0x16);
const AMBER: Color = Color::srgb_u8(0xff, 0xb0, 0x00);

const JOB_SECS: f32 = 1.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, cpu.spawn())
        .add_systems(Update, (work, settle));
}

#[derive(Component, FromTemplate)]
struct Cpu {
    /// Running job, pays 1$ when finished.
    job: Option<Timer>,
    /// Progress bar fill showing job progress.
    fill: Entity,
}

/// Clickable DIP-40 CPU with a progress bar below.
fn cpu() -> impl Scene {
    let pins = (0..40)
        .map(|i| rect(PIN, 10., 24., -190. + 20. * (i % 20) as f32, if i < 20 { 70. } else { -70. }, -0.1))
        .collect::<Vec<_>>();
    bsn! {
        #Cpu
        Cpu { fill: #ProgressFill }
        @rect(BODY, 420., 130., 0., 0., 0.)
        Pickable
        on(click)
        Children [
            @rect(LID, 150., 80., 0., 0., 0.1)
            --
            @rect(DARK, 14., 34., -203., 0., 0.1)
            --
            @rect(DARK, 10., 10., -185., -45., 0.1)
            --
            #ProgressBar
            @rect(DARK, 420., 12., 0., -110., 0.)
            --
            #ProgressFill
            @rect(AMBER, 420., 12., -210., -110., 0.1)
            Anchor::CENTER_LEFT
            Transform { scale: {vec3(0., 1., 1.)} }
            --
            {pins}
        ]
    }
}

/// Solid `w`×`h` rectangle at `x`, `y`, `z`.
fn rect(color: Color, w: f32, h: f32, x: f32, y: f32, z: f32) -> impl Scene {
    bsn! {
        Sprite { color, custom_size: {Some(vec2(w, h))} }
        Transform { translation: {vec3(x, y, z)} }
    }
}

/// Squashes the CPU, ticks and starts a job unless one is running.
fn click(mut click: On<PointerClick>, mut commands: Commands, mut cpus: Query<(&mut Cpu, &mut Transform)>) -> Result {
    click.propagate(false);
    let (mut cpu, mut transform) = cpus.get_mut(click.entity)?;
    transform.scale = Vec3::splat(0.9);
    commands.spawn_scene(bsn! { AudioPlayer("audio/click.wav") PlaybackSettings::DESPAWN });
    cpu.job.get_or_insert_with(|| Timer::from_seconds(JOB_SECS, TimerMode::Once));
    Ok(())
}

fn work(
    time: Res<Time>,
    mut commands: Commands,
    mut currencies: ResMut<Currencies>,
    mut cpus: Query<&mut Cpu>,
    mut transforms: Query<&mut Transform>,
) -> Result {
    for mut cpu in &mut cpus {
        let fill = cpu.fill;
        let Some(job) = &mut cpu.job else { continue };
        let done = job.tick(time.delta()).is_finished();
        transforms.get_mut(fill)?.scale.x = if done { 0. } else { job.fraction() };
        if done {
            cpu.job = None;
            currencies.usd += 1;
            commands.spawn_scene(usd_pop());
        }
    }
    Ok(())
}

/// Eases squashed CPUs back to normal size.
fn settle(time: Res<Time>, mut cpus: Query<&mut Transform, With<Cpu>>) {
    for mut transform in &mut cpus {
        transform.scale.smooth_nudge(&Vec3::ONE, 12., time.delta_secs());
    }
}
