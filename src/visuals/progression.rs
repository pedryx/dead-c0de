use bevy::prelude::*;

use super::{AMBER, BEZEL, GREEN, SCREEN, shop::Reveal};
use crate::game::progression::{GOAL_USD, Progression};

const CLICK_HINT: &str = "click the CPU to run its program";
const PAY_HINT: &str = "each run pays out the program's result";
/// How long a hint takes to fade in or out, in seconds.
const FADE_SECONDS: f32 = 0.5;
/// Characters typed per second.
const TYPING_SPEED: f32 = 40.;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (show_hints, type_text, close_victory));
}

/// Text typed out one character at a time.
#[derive(Component, Default, Clone)]
struct Typewriter {
    text: String,
    /// Seconds spent typing.
    seconds: f32,
}

/// Goal at the top of the screen.
pub fn goal_visuals() -> impl Scene {
    let goal = format!("earn {GOAL_USD}$ to ship 0xDEAD_C0DE");
    bsn! {
        Reveal
        Node {
            position_type: PositionType::Absolute,
            top: px(20.),
            width: percent(100.),
            justify_content: JustifyContent::Center,
        }
        Pickable::IGNORE
        Children [ Text(goal) TextFont { font_size: FontSize::Px(24.) } TextColor(AMBER) Pickable::IGNORE ]
    }
}

/// Shipping report popup, for winning `seconds` after the start.
pub fn victory_visuals(seconds: f32) -> impl Scene {
    let report = format!(
        "compiling... 9001 warnings, 0 errors\n\
         running tests... skipped, works on my machine\n\
         burning floppies... disk 37 of 37\n\
         uploading at 56 kbit/s... done\n\
         {GOAL_USD}$ earned in {}:{:02}\n\n\
         YOU WON!\n\
         0xDEAD_C0DE has performed a LEGAL operation\n\
         and will be shipped.\n\n\
         press ANY KEY to keep grinding",
        seconds as u32 / 60,
        seconds as u32 % 60,
    );
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.),
            height: percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            padding: UiRect::top(percent(12.)),
        }
        BackgroundColor({SCREEN.with_alpha(0.8)})
        GlobalZIndex(10)
        AudioPlayer("audio/win.wav")
        Children [
            Node { flex_direction: FlexDirection::Column, width: px(600.), border: UiRect::all(px(4.)) }
            BackgroundColor(SCREEN)
            BorderColor::all(BEZEL)
            Children [
                Node { padding: UiRect::axes(px(10.), px(6.)) }
                BackgroundColor(BEZEL)
                Children [ Text("root@c0de:~$ ./ship_it.sh") TextFont { font_size: FontSize::Px(16.) } TextColor(AMBER) ]
                --
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(20.),
                    padding: UiRect::all(px(20.)),
                }
                Children [
                    Text
                    Typewriter { text: report }
                    TextFont { font_size: FontSize::Px(20.) }
                    TextColor(GREEN)
                    Node { align_self: AlignSelf::Stretch }
                    --
                    Node { padding: UiRect::axes(px(16.), px(8.)) }
                    BackgroundColor(BEZEL)
                    Children [ Text("[ ANY KEY ]") TextFont { font_size: FontSize::Px(20.) } TextColor(AMBER) Pickable::IGNORE ]
                ]
            ]
        ]
    }
}

/// Types out texts.
fn type_text(time: Res<Time>, mut texts: Query<(&mut Typewriter, &mut Text)>) {
    for (mut typewriter, mut text) in &mut texts {
        typewriter.seconds += time.delta_secs();
        let typed = ((typewriter.seconds * TYPING_SPEED) as usize).min(typewriter.text.len());
        if text.0.len() != typed {
            text.0 = typewriter.text[..typed].into();
        }
    }
}

/// Closes the victory popup.
fn close_victory(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    reports: Query<Entity, With<Typewriter>>,
    parents: Query<&ChildOf>,
) {
    if keys.get_just_pressed().next().is_none() && mouse.get_just_pressed().next().is_none() {
        return;
    }
    for report in &reports {
        if let Some(popup) = parents.iter_ancestors(report).last() {
            commands.entity(popup).despawn();
        }
    }
}

/// Hint above the starting CPU.
pub fn progression_visuals() -> impl Scene {
    bsn! {
        Text2d
        TextFont { font_size: FontSize::Px(20.) }
        TextColor({AMBER.with_alpha(0.)})
        Transform { translation: vec3(0., 80., 1.) }
    }
}

/// Writes the hint for the progress, fades it in, or out once passed, and pulses it.
fn show_hints(
    time: Res<Time>,
    mut hints: Query<(&Progression, &mut Text2d, &mut TextColor, &mut Transform)>,
) {
    let seconds = time.elapsed_secs();
    for (progression, mut text, mut color, mut transform) in &mut hints {
        transform.scale = Vec3::splat(1. + 0.05 * (seconds * 4.).sin());
        let (hint, shown) = match progression {
            Progression::Start => (CLICK_HINT, true),
            Progression::FirstRun => (CLICK_HINT, false),
            Progression::FirstPay => (PAY_HINT, true),
            Progression::SecondPay => (PAY_HINT, false),
        };
        if text.0 != hint {
            text.0 = hint.into();
        }
        let fade = time.delta_secs() / FADE_SECONDS * if shown { 1. } else { -1. };
        let alpha = (color.0.alpha() + fade).clamp(0., 1.);
        color.0.set_alpha(alpha);
    }
}
