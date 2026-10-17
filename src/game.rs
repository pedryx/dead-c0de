use bevy::prelude::*;

#[cfg(feature = "dev")]
mod cheats;
mod cpu;
mod currency;
mod particles;
mod shop;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((cpu::plugin, currency::plugin, particles::plugin, shop::plugin));
    #[cfg(feature = "dev")]
    app.add_plugins(cheats::plugin);
}
