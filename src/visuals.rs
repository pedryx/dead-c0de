use bevy::{audio::Volume, prelude::*, sprite::Anchor};

mod background;
pub mod board;
#[cfg(feature = "dev")]
pub mod cheats;
pub mod cpu;
pub mod currencies;
pub mod particles;
pub mod shop;

const RUST: Color = hex("#8b3a1a");
const BEZEL: Color = hex("#3b3430");
const SCREEN: Color = hex("#061208");
const ROW: Color = hex("#0c2212");
const GREEN: Color = hex("#33ff66");
const DIM: Color = hex("#1f7a3a");
const AMBER: Color = hex("#ffb000");

pub fn plugin(app: &mut App) {
    app.add_plugins((
        background::plugin,
        board::plugin,
        cpu::plugin,
        currencies::plugin,
        particles::plugin,
        shop::plugin,
    ));
}

/// Color from a `#rrggbb` or `#rrggbbaa` code.
const fn hex(code: &str) -> Color {
    let Ok(value) = u32::from_str_radix(code.split_at(1).1, 16) else {
        panic!("invalid hex color");
    };
    let [red, green, blue, alpha] = if code.len() == 9 {
        value
    } else {
        value << 8 | 0xff
    }
    .to_be_bytes();
    Color::srgba_u8(red, green, blue, alpha)
}

/// Plays the sound at `path` once.
fn sound(path: &'static str) -> impl Scene {
    sound_with_volume(path, 1.)
}

/// Plays the sound at `path` once at `volume`, where 1 is full volume.
fn sound_with_volume(path: &'static str, volume: f32) -> impl Scene {
    bsn! { AudioPlayer(path) PlaybackSettings::DESPAWN PlaybackSettings { volume: Volume::Linear(volume) } }
}

/// Solid rectangle of `size` centered at `position`.
fn rectangle(color: Color, position: Vec3, size: Vec2) -> impl Scene {
    bsn! {
        Sprite { color, custom_size: {Some(size)} }
        Transform { translation: position }
    }
}

/// Left-aligned `font_size`px text at `position`.
fn text(text: impl Into<String>, color: Color, font_size: f32, position: Vec2) -> impl Scene {
    let text = text.into();
    bsn! {
        Text2d(text)
        TextFont { font_size: FontSize::Px(font_size) }
        TextColor(color)
        Anchor::CENTER_LEFT
        Transform { translation: {position.extend(0.5)} }
    }
}
