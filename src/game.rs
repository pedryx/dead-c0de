use bevy::prelude::*;

pub mod board;
#[cfg(feature = "dev")]
mod cheats;
pub mod cpu;
pub mod currencies;
pub mod shop;

pub fn plugin(app: &mut App) {
    app.add_plugins((board::plugin, cpu::plugin, currencies::plugin, shop::plugin));
    #[cfg(feature = "dev")]
    app.add_plugins(cheats::plugin);
}
