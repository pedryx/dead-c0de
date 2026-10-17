use bevy::prelude::*;

use super::{
    cpu::{AMBER, BEZEL, RUST, SCREEN},
    currency::Currencies,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, cheats.spawn());
}

/// Dev-only cheat bar centered at the bottom of the screen.
fn cheats() -> impl Scene {
    bsn! {
        #Cheats
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
                padding: {UiRect::all(px(6.))},
                border: {UiRect::all(px(2.))},
            }
            BackgroundColor(SCREEN)
            ~{BorderColor::all(RUST)}
            Children [
                Text("CHEATS") TextFont { font_size: FontSize::Px(14.) } TextColor(RUST)
                --
                Node { padding: {UiRect::axes(px(10.), px(4.))} }
                BackgroundColor(BEZEL)
                on(rich)
                Children [ Text("+1M$") TextFont { font_size: FontSize::Px(16.) } TextColor(AMBER) Pickable::IGNORE ]
            ]
        ]
    }
}

fn rich(_: On<PointerClick>, mut currencies: ResMut<Currencies>) {
    currencies.usd += 1_000_000;
}
