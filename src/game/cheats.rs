use bevy::prelude::*;

use super::currencies::Currencies;
use crate::visuals::cheats::{cheat_button_visuals, cheats_visuals};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, cheats.spawn());
}

fn cheats() -> impl Scene {
    let buttons = bsn_list! { @cheat_button_visuals("+1M$") on(add_million) };
    bsn! { #Cheats @cheats_visuals(buttons) }
}

fn add_million(_: On<PointerClick>, mut currencies: Single<&mut Currencies>) {
    currencies.usd += 1_000_000;
}
