use bevy::prelude::*;

use super::{AMBER, BEZEL, RUST, SCREEN};

/// Cheat bar at the bottom of the screen.
pub fn cheats_visuals(buttons: impl SceneList) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            bottom: px(16.),
            left: px(0.),
            right: px(0.),
            justify_content: JustifyContent::Center,
        }
        Pickable::IGNORE
        Children [
            Node {
                align_items: AlignItems::Center,
                column_gap: px(8.),
                padding: UiRect::all(px(6.)),
                border: UiRect::all(px(2.)),
            }
            BackgroundColor(SCREEN)
            BorderColor::all(RUST)
            Children [
                Text("CHEATS") TextFont { font_size: FontSize::Px(14.) } TextColor(RUST)
                --
                {buttons}
            ]
        ]
    }
}

/// Cheat button showing `label`.
pub fn cheat_button_visuals(label: &'static str) -> impl Scene {
    bsn! {
        Node { padding: UiRect::axes(px(10.), px(4.)) }
        BackgroundColor(BEZEL)
        Children [ Text(label) TextFont { font_size: FontSize::Px(16.) } TextColor(AMBER) Pickable::IGNORE ]
    }
}
