use bevy::prelude::*;

use super::AMBER;
use crate::game::progression::Progression;

const CLICK_HINT: &str = "click the CPU to run its program";
const PAY_HINT: &str = "each run pays out the program's result";
/// How long a hint takes to fade in or out, in seconds.
const FADE_SECONDS: f32 = 0.5;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, show_hints);
}

/// Hint above the starting CPU.
pub fn progression_visuals() -> impl Scene {
    bsn! {
        Text2d
        TextFont { font_size: FontSize::Px(20.) }
        TextColor({AMBER.with_alpha(0.)})
        Transform { translation: vec3(0., 80., 1.) }
    }
}

/// Writes the hint for the progress, fades it in, or out once passed, and pulses it.
fn show_hints(
    time: Res<Time>,
    mut hints: Query<(&Progression, &mut Text2d, &mut TextColor, &mut Transform)>,
) {
    let seconds = time.elapsed_secs();
    for (progression, mut text, mut color, mut transform) in &mut hints {
        transform.scale = Vec3::splat(1. + 0.05 * (seconds * 4.).sin());
        let (hint, shown) = match progression {
            Progression::Start => (CLICK_HINT, true),
            Progression::FirstRun => (CLICK_HINT, false),
            Progression::FirstPay => (PAY_HINT, true),
            Progression::SecondPay => (PAY_HINT, false),
        };
        if text.0 != hint {
            text.0 = hint.into();
        }
        let fade = time.delta_secs() / FADE_SECONDS * if shown { 1. } else { -1. };
        let alpha = (color.0.alpha() + fade).clamp(0., 1.);
        color.0.set_alpha(alpha);
    }
}
