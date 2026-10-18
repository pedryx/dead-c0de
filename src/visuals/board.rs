use bevy::prelude::*;

use super::{particles::burst, rectangle, sound};
use crate::game::board::Board;

/// Opacity of the ghost CPU.
const GHOST_ALPHA: f32 = 0.35;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, fade_ghost);
}

/// Invisible background catching the pointer outside the shops.
pub fn board_visuals() -> impl Scene {
    bsn! {
        Transform
        Visibility
        Children [ @rectangle(Color::NONE, vec3(0., 0., -10.), Vec2::splat(100_000.)) Pickable on(burst) ]
    }
}

/// Fades a new ghost and plays a beep.
fn fade_ghost(
    mut commands: Commands,
    boards: Query<&Board, Changed<Board>>,
    children: Query<&Children>,
    mut sprites: Query<&mut Sprite>,
    mut texts: Query<&mut TextColor>,
) {
    for ghost in boards.iter().filter_map(|board| board.ghost) {
        commands.spawn_scene(sound("audio/target.wav"));
        for entity in children.iter_descendants(ghost) {
            if let Ok(mut sprite) = sprites.get_mut(entity) {
                sprite.color = sprite.color.with_alpha(sprite.color.alpha() * GHOST_ALPHA);
            }
            if let Ok(mut color) = texts.get_mut(entity) {
                color.0 = color.0.with_alpha(color.0.alpha() * GHOST_ALPHA);
            }
        }
    }
}
