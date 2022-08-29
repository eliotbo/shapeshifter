use bevy::prelude::*;

use super::{despawn_screen, GameState};

use shapeshifter_level_maker::{input::Action, util::SpawnPolyKeepPoly};

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Splash).with_system(splash_setup))
            .add_system_set(SystemSet::on_update(GameState::Splash).with_system(countdown))
            .add_system_set(
                SystemSet::on_exit(GameState::Splash).with_system(despawn_screen::<OnSplashScreen>),
            );
    }
}

#[derive(Component)]
struct OnSplashScreen;

#[derive(Deref, DerefMut)]
struct SplashTimer(Timer);

#[derive(Deref, DerefMut)]
struct CutTimer1(Timer);

#[derive(Deref, DerefMut)]
struct CutTimer2(Timer);

struct LogoSoundsHandle {
    cut: Handle<AudioSource>,
    // logo: Handle<AudioSource>,
}

const LOGO_TIME: f32 = 5.0;
const CUT_TIME: f32 = 3.0;
const CUTTING_TIME: f32 = 0.05;

fn splash_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut spawn_polykeep_event_writer: EventWriter<SpawnPolyKeepPoly>,
) {
    // let icon = asset_server.load("branding/icon.png");
    let scissor2 = asset_server.load("sounds/Scissor Cut/Scissor paper cut 2.ogg");
    let logo_sound = asset_server.load("sounds/Shapeshift Logo v2.ogg");

    audio.play(logo_sound.clone());

    commands.insert_resource(LogoSoundsHandle {
        cut: scissor2,
        // logo: logo_sound,
    });

    // commands
    //     .spawn_bundle(ImageBundle {
    //         style: Style {
    //             margin: UiRect::all(Val::Auto),
    //             size: Size::new(Val::Px(1000.0), Val::Auto),
    //             ..default()
    //         },
    //         image: UiImage(icon),
    //         ..default()
    //     })
    //     .insert(OnSplashScreen);

    commands.insert_resource(SplashTimer(Timer::from_seconds(LOGO_TIME, false)));
    commands.insert_resource(CutTimer1(Timer::from_seconds(CUT_TIME, false)));
    commands.insert_resource(CutTimer2(Timer::from_seconds(1111111.0, false)));

    let s = 0.35;
    for (letter, translation, scale) in [
        ("ss", Vec3::new(-400.0, 0.0, 0.1), s),
        ("hh", Vec3::new(-300.0, 0.0, 0.11), s),
        ("aa", Vec3::new(-200.0, -20.0, 0.111), 0.37),
        ("pp", Vec3::new(-100.0, 0.0, 0.1111), s),
        ("ee", Vec3::new(-00.0, -3.0, 0.2), 0.32),
        ("ss", Vec3::new(100.0, -3.0, 0.22), s),
        ("hh", Vec3::new(200.0, 0.0, 0.12), s),
        ("ii", Vec3::new(270.0, -5.0, 0.13), 0.25),
        ("ff", Vec3::new(340.0, 5.0, 0.14), 0.3),
        ("tt", Vec3::new(430.0, 2.0, 0.15), 0.3),
    ] {
        let mut transform = Transform::from_translation(translation);
        transform.scale = Vec3::splat(scale);
        spawn_polykeep_event_writer.send(SpawnPolyKeepPoly {
            polygon: letter.to_string(),
            polygon_multiplier: 1.0,
            maybe_transform: Some(transform),
        });
    }
}

fn countdown(
    mut game_state: ResMut<State<GameState>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
    mut cut_timer1: ResMut<CutTimer1>,
    mut cut_timer2: ResMut<CutTimer2>,
    logo_sounds: Res<LogoSoundsHandle>,
    audio: Res<Audio>,
    mut action_event_writer: EventWriter<shapeshifter_level_maker::input::Action>,
) {
    if timer.tick(time.delta()).finished() {
        game_state.set(GameState::Menu).unwrap();
    }

    let x = 800.0;
    let y = 20.0;

    // start the cut
    if cut_timer1.tick(time.delta()).just_finished() {
        //
        //
        // let start = Vec2::new(-x, -y);
        let start = Vec2::new(10., -10.);

        action_event_writer
            .send(shapeshifter_level_maker::input::Action::StartMakingCutSegment { start });

        // new timer for end of cut
        *cut_timer2 = CutTimer2(Timer::from_seconds(CUTTING_TIME, false));
    }

    // end the cut
    if cut_timer2.tick(time.delta()).just_finished() {
        //
        //
        let end = Vec2::new(x + 100.0, y);

        action_event_writer.send(shapeshifter_level_maker::input::Action::EndCutSegment { end });

        audio.play(logo_sounds.cut.clone());
    }
}
