use bevy::prelude::*;

const GREEN: Color = Color::srgb_u8(0x5c, 0xe0, 0x5c);

const POP_SECS: f32 = 0.8;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Currencies>()
        .add_systems(Startup, usd_counter.spawn())
        .add_systems(Update, (float_usd, show_usd.run_if(resource_changed::<Currencies>)));
}

/// Money the player owns.
#[derive(Resource, Default)]
pub struct Currencies {
    pub usd: u64,
}

/// Floating $ icon, holds its age in seconds.
#[derive(Component, Default, Clone)]
struct UsdPop(f32);

/// Text showing [`Currencies::usd`].
#[derive(Component, Default, Clone)]
struct UsdCounter;

fn usd_counter() -> impl Scene {
    bsn! {
        #UsdCounter
        UsdCounter
        Text
        TextFont { font_size: FontSize::Px(32.) }
        TextColor(GREEN)
        Node { position_type: PositionType::Absolute, top: px(16.), right: px(16.) }
    }
}

/// Chiming $ icon that rises, wobbles and fades out.
pub(super) fn usd_pop() -> impl Scene {
    bsn! {
        #UsdPop
        UsdPop
        AudioPlayer("audio/coin.wav")
        Text2d("$")
        TextFont { font_size: FontSize::Px(48.) }
        TextColor(GREEN)
        Transform { translation: {vec3(0., 90., 1.)} }
    }
}

fn float_usd(
    time: Res<Time>,
    mut commands: Commands,
    mut pops: Query<(Entity, &mut UsdPop, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut pop, mut transform, mut color) in &mut pops {
        pop.0 += time.delta_secs();
        let t = pop.0 / POP_SECS;
        transform.translation = vec3((pop.0 * 18.).sin() * 12., 90. + t * 120., 1.);
        color.0 = GREEN.with_alpha(1. - t);
        if t >= 1. {
            commands.entity(entity).despawn();
        }
    }
}

fn show_usd(currencies: Res<Currencies>, mut counter: Single<&mut Text, With<UsdCounter>>) {
    counter.0 = format!("{}$", currencies.usd);
}
