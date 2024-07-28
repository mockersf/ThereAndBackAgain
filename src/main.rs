#![windows_subsystem = "windows"]

use avian3d::prelude::*;
#[cfg(feature = "debug")]
use bevy::window::PresentMode;
use bevy::{
    asset::{embedded_asset, AssetMetaCheck},
    core_pipeline::bloom::BloomSettings,
    prelude::*,
};
use bevy_easings::EasingsPlugin;
use bevy_firework::plugin::ParticleSystemPlugin;

use bevy_pkv::PkvStore;
use there_and_back_again::{
    audio, credits, game, level_selector, levels, loading, lost, menu, play, win, GameProgress,
    GameState,
};

fn main() {
    let mut app = App::new();

    // needed for bevy_firework on web
    app.insert_resource(Msaa::Off);

    let store = PkvStore::new("Vleue", "ThereAndBackAgain");
    let game_progress = GameProgress {
        current_level: if cfg!(feature = "debug") {
            usize::MAX
        } else {
            store.get::<u32>("progress").unwrap_or(1) as usize
        },
    };
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "There And Back Again".to_string(),
                    #[cfg(feature = "debug")]
                    present_mode: PresentMode::AutoNoVsync,
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .init_state::<GameState>()
    .enable_state_scoped_entities::<GameState>()
    .add_plugins((
        EasingsPlugin,
        PhysicsPlugins::default(),
        ParticleSystemPlugin,
    ))
    .insert_resource(store)
    .add_plugins((
        loading::Plugin,
        menu::Plugin,
        levels::Plugin,
        credits::Plugin,
        game::Plugin,
        level_selector::Plugin,
        play::Plugin,
        win::Plugin,
        lost::Plugin,
        audio::Plugin,
    ))
    .add_systems(Startup, camera);

    app.insert_resource(game_progress);

    #[cfg(feature = "debug")]
    app.add_plugins(PhysicsDebugPlugin::default());

    embedded_asset!(app, "branding/logo.png");
    embedded_asset!(app, "branding/bevy_logo_dark.png");
    embedded_asset!(app, "branding/birdoggo.png");

    app.world_mut().spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        BloomSettings::NATURAL,
    ));

    app.run();
}

fn camera(mut commands: Commands) {
    commands.spawn((DirectionalLightBundle {
        transform: Transform::IDENTITY.looking_to(Vec3::new(1.0, -1.0, 1.0), Vec3::Y),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: light_consts::lux::OVERCAST_DAY * 2.0,
            ..default()
        },
        ..default()
    },));
}
