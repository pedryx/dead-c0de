use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use rand::Rng;

use super::{hex, rectangle};

const TRACE: Color = hex("#4f3222");
const PAD: Color = hex("#573826");
const SIGNAL: Color = hex("#c46a2b66");
const TRACE_WIDTH: f32 = 4.;
const PAD_SIZE: f32 = 10.;
const SIGNAL_SIZE: f32 = 6.;
/// Spacing of the grid traces run on, in pixels.
const GRID: f32 = 40.;
/// Area around the center where traces start, in pixels.
const AREA: Vec2 = vec2(3200., 1800.);
const TRACES: usize = 80;
/// How fast signals run along traces, in pixels per second.
const SIGNAL_SPEED: f32 = 60.;
/// Seconds a signal waits before running its trace again.
const SIGNAL_PAUSE: f32 = 20.;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, background.spawn())
        .add_systems(Update, run_signals);
}

/// Light running along a trace.
#[derive(Component, Clone)]
struct Signal {
    path: Vec<Vec2>,
    /// Distance from the trace start, in pixels.
    traveled: f32,
}

/// Old circuit board as background.
fn background() -> impl Scene {
    let traces: Vec<_> = (0..TRACES).map(|_| trace()).collect();
    bsn! { #Background Transform Visibility Children [ {traces} ] }
}

/// Random trace with a pad on each end and a signal running on it.
fn trace() -> impl Scene {
    let mut rng = rand::rng();
    let mut point = ((vec2(rng.random(), rng.random()) - 0.5) * AREA / GRID).round() * GRID;
    let mut heading: i32 = rng.random_range(0..8);
    let mut path = vec![point];
    for _ in 0..rng.random_range(2..6) {
        point += Vec2::from_angle(heading as f32 * FRAC_PI_4).round()
            * GRID
            * rng.random_range(2..8) as f32;
        path.push(point);
        heading += if rng.random_bool(0.5) { 1 } else { -1 };
    }
    let segments: Vec<_> = path
        .windows(2)
        .map(|ends| segment(ends[0], ends[1]))
        .collect();
    let pads: Vec<_> = [path[0], point]
        .into_iter()
        .map(|point| rectangle(PAD, point.extend(-19.), Vec2::splat(PAD_SIZE)))
        .collect();
    let signal = Signal {
        path,
        traveled: rng.random_range(0.0..SIGNAL_PAUSE * SIGNAL_SPEED),
    };
    bsn! {
        Transform
        Visibility
        Children [
            {segments}
            --
            {pads}
            --
            signal
            @rectangle(SIGNAL, Vec3::ZERO, Vec2::splat(SIGNAL_SIZE))
        ]
    }
}

/// Straight piece of trace from `start` to `end`.
fn segment(start: Vec2, end: Vec2) -> impl Scene {
    let center = start.midpoint(end).extend(-20.);
    let size = vec2(start.distance(end) + TRACE_WIDTH, TRACE_WIDTH);
    let rotation = Quat::from_rotation_z((end - start).to_angle());
    bsn! { @rectangle(TRACE, center, size) Transform { rotation } }
}

/// Moves signals along their traces and hides them while they wait.
fn run_signals(
    time: Res<Time>,
    mut signals: Query<(&mut Signal, &mut Transform, &mut Visibility)>,
) {
    for (mut signal, mut transform, mut visibility) in &mut signals {
        signal.traveled += SIGNAL_SPEED * time.delta_secs();
        let mut left = signal.traveled;
        for ends in signal.path.windows(2) {
            transform.translation = ends[0].move_towards(ends[1], left).extend(-18.);
            left -= ends[0].distance(ends[1]);
            if left <= 0. {
                break;
            }
        }
        *visibility = if left > 0. {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if left > SIGNAL_PAUSE * SIGNAL_SPEED {
            signal.traveled = 0.;
        }
    }
}
