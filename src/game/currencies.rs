use bevy::prelude::*;

use crate::visuals::currencies::currencies_visuals;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, currencies.spawn());
}

/// Money the player owns.
#[derive(Component, Default, Clone)]
pub struct Currencies {
    pub usd: u64,
}

fn currencies() -> impl Scene {
    bsn! {
        #Currencies
        Currencies
        @currencies_visuals()
    }
}
