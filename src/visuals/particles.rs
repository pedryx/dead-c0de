use std::f32::consts::PI;

use bevy::prelude::*;
use rand::{Rng, seq::IndexedRandom};

use super::{AMBER, GREEN, hex, rectangle, sound_with_volume};

const BIT_COLORS: [Color; 2] = [GREEN, AMBER];
const FLAKE_COLORS: [Color; 4] = [
    hex("#6e2f14"),
    hex("#8b3a1a"),
    hex("#a0522d"),
    hex("#c46a2b"),
];
/// How long a particle lasts, in seconds.
const LIFETIME: f32 = 0.5;
/// How strongly particles fall, in pixels per second squared.
const GRAVITY: f32 = 300.;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, fly);
}

/// Bit or rust flake thrown out by a background click.
#[derive(Component, Default, Clone)]
struct Particle {
    velocity: Vec2,
    /// Rotation speed in radians per second.
    spin: f32,
    /// Seconds since spawn.
    age: f32,
}

/// Throws out particles and plays a zap where the background was clicked.
pub fn burst(click: On<PointerClick>, mut commands: Commands) {
    let Some(point) = click.hit.position else {
        return;
    };
    let mut rng = rand::rng();
    let speed: f32 = rng.random_range(0.8..1.25);
    commands
        .spawn_scene(bsn! { @sound_with_volume("audio/zap.wav", 0.25) PlaybackSettings { speed } });
    for _ in 0..rng.random_range(2..5) {
        commands.spawn_scene(particle(point, &mut rng));
    }
}

/// Random particle at `point`, flying up.
fn particle(point: Vec3, rng: &mut impl Rng) -> impl Scene {
    let velocity = Vec2::from_angle(rng.random_range(0.2..PI - 0.2)) * rng.random_range(50.0..120.);
    let spin: f32 = rng.random_range(-10.0..10.);
    let look: Box<dyn Scene> = if rng.random_bool(0.5) {
        let (bit, color) = (
            if rng.random_bool(0.5) { "0" } else { "1" },
            *BIT_COLORS.choose(rng).unwrap(),
        );
        Box::new(bsn! { Text2d(bit) TextFont { font_size: FontSize::Px(12.) } TextColor(color) })
    } else {
        let (size, color) = (
            rng.random_range(2.0..4.),
            *FLAKE_COLORS.choose(rng).unwrap(),
        );
        Box::new(rectangle(color, Vec3::ZERO, Vec2::splat(size)))
    };
    bsn! {
        Particle { velocity, spin }
        @look
        Transform::from_translation(point.with_z(5.))
    }
}

/// Moves, spins and fades out particles.
fn fly(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(
        Entity,
        &mut Particle,
        &mut Transform,
        Option<&mut Sprite>,
        Option<&mut TextColor>,
    )>,
) {
    let delta = time.delta_secs();
    for (entity, mut particle, mut transform, sprite, text) in &mut particles {
        particle.age += delta;
        particle.velocity.y -= GRAVITY * delta;
        transform.translation += (particle.velocity * delta).extend(0.);
        transform.rotate_z(particle.spin * delta);
        let alpha = 1. - particle.age / LIFETIME;
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
