use std::f32::consts::PI;

use bevy::{audio::Volume, prelude::*};
use rand::{Rng, seq::IndexedRandom};

const BITS: [Color; 2] = [Color::srgb_u8(0x5c, 0xe0, 0x5c), Color::srgb_u8(0xff, 0xb0, 0x00)];
const RUST: [Color; 4] = [
    Color::srgb_u8(0x6e, 0x2f, 0x14),
    Color::srgb_u8(0x8b, 0x3a, 0x1a),
    Color::srgb_u8(0xa0, 0x52, 0x2d),
    Color::srgb_u8(0xc4, 0x6a, 0x2b),
];

const LIFE: f32 = 0.5;
const GRAVITY: f32 = 300.;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(burst).add_systems(Update, fly);
}

/// Short-lived particle flung out by a background click.
#[derive(Component, Default, Clone)]
struct Particle {
    velocity: Vec2,
    /// Rotation speed in radians per second.
    spin: f32,
    /// Seconds since spawn.
    age: f32,
}

/// Bursts bits and rust flakes with a randomly pitched zap where the background was clicked.
fn burst(
    click: On<PointerClick>,
    windows: Query<(), With<Window>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
) -> Result {
    if !windows.contains(click.entity) {
        return Ok(());
    }
    let at = camera.0.viewport_to_world_2d(camera.1, click.pointer.position)?;
    let mut rng = rand::rng();
    let speed: f32 = rng.random_range(0.8..1.25);
    commands.spawn_scene(bsn! {
        AudioPlayer("audio/zap.wav")
        PlaybackSettings::DESPAWN
        PlaybackSettings { speed, volume: Volume::Linear(0.08) }
    });
    for _ in 0..rng.random_range(2..5) {
        commands.spawn_scene(particle(at, &mut rng));
    }
    Ok(())
}

/// Random binary digit or rust flake at `at`, flying upward.
fn particle(at: Vec2, rng: &mut impl Rng) -> impl Scene {
    let velocity = Vec2::from_angle(rng.random_range(0.2..PI - 0.2)) * rng.random_range(50.0..120.);
    let spin: f32 = rng.random_range(-10.0..10.);
    let look: Box<dyn Scene> = if rng.random_bool(0.5) {
        let (bit, color) = (if rng.random_bool(0.5) { "0" } else { "1" }, *BITS.choose(rng).unwrap());
        Box::new(bsn! { Text2d(bit) TextFont { font_size: FontSize::Px(12.) } TextColor(color) })
    } else {
        let (size, color) = (rng.random_range(2.0..4.), *RUST.choose(rng).unwrap());
        Box::new(bsn! { Sprite { color, custom_size: {Some(Vec2::splat(size))} } })
    };
    bsn! {
        Particle { velocity, spin }
        Transform { translation: {at.extend(5.)} }
        @look
    }
}

fn fly(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Particle, &mut Transform, Option<&mut Sprite>, Option<&mut TextColor>)>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut transform, sprite, text) in &mut particles {
        particle.age += dt;
        particle.velocity.y -= GRAVITY * dt;
        transform.translation += (particle.velocity * dt).extend(0.);
        transform.rotate_z(particle.spin * dt);
        let alpha = 1. - particle.age / LIFE;
        if let Some(mut sprite) = sprite {
            sprite.color.set_alpha(alpha);
        }
        if let Some(mut text) = text {
            text.0.set_alpha(alpha);
        }
        if alpha <= 0. {
            commands.entity(entity).despawn();
        }
    }
}
