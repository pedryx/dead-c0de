use bevy::prelude::*;

use super::GREEN;
use crate::game::currencies::Currencies;

/// How long a money pop lasts, in seconds.
const POP_LIFETIME: f32 = 0.8;
/// How fast a money pop rises, in pixels per second.
const POP_RISE_SPEED: f32 = 150.;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (show_usd, float_pops));
}

/// Seconds since the money pop appeared.
#[derive(Component, Default, Clone)]
struct Pop(f32);

/// USD counter in the top right corner.
pub fn currencies_visuals() -> impl Scene {
    bsn! {
        Text
        TextFont { font_size: FontSize::Px(32.) }
        TextColor(GREEN)
        Node { position_type: PositionType::Absolute, top: px(16.), right: px(16.) }
    }
}

/// Floating `+usd$` text with a coin sound.
pub fn usd_pop(usd: u64) -> impl Scene {
    let text = format!("+{usd}$");
    bsn! {
        Pop
        AudioPlayer("audio/coin.wav")
        Text2d(text)
        TextFont { font_size: FontSize::Px(28.) }
        TextColor(GREEN)
        Transform::from_xyz(0., 30., 1.)
    }
}

/// Updates the money counter.
fn show_usd(mut counters: Query<(&Currencies, &mut Text), Changed<Currencies>>) {
    for (currencies, mut text) in &mut counters {
        text.0 = format!("{}$", currencies.usd);
    }
}

/// Moves money pops up and fades them out.
fn float_pops(
    time: Res<Time>,
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Pop, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut pop, mut transform, mut color) in &mut pops {
        pop.0 += time.delta_secs();
        transform.translation.x = (pop.0 * 18.).sin() * 12.;
        transform.translation.y += POP_RISE_SPEED * time.delta_secs();
        color.0 = GREEN.with_alpha(1. - pop.0 / POP_LIFETIME);
        if pop.0 >= POP_LIFETIME {
            commands.entity(entity).despawn();
        }
    }
}
