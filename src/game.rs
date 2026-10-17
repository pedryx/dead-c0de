use bevy::prelude::*;

mod cpu;
mod currency;
mod particles;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((cpu::plugin, currency::plugin, particles::plugin));
}
