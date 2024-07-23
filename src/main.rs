#![windows_subsystem = "windows"]

use avian3d::prelude::*;
#[cfg(feature = "debug")]
use bevy::window::PresentMode;
use bevy::{
    asset::{embedded_asset, AssetMetaCheck},
    prelude::*,
};
use bevy_easings::EasingsPlugin;
use bevy_firework::plugin::ParticleSystemPlugin;
use levels::DirectionalLightIlluminance;

mod assets;
mod credits;
mod game;
mod levels;
mod loading;
mod menu;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    Menu,
    Credits,
}

fn main() {
    let mut app = App::new();

    // needed for bevy_firework on web
    app.insert_resource(Msaa::Off);

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
    .add_plugins((
        loading::Plugin,
        menu::Plugin,
        levels::Plugin,
        credits::Plugin,
        game::Plugin,
    ))
    .add_systems(Startup, camera);

    #[cfg(feature = "debug")]
    app.add_plugins(PhysicsDebugPlugin::default());

    embedded_asset!(app, "branding/logo.png");
    embedded_asset!(app, "branding/bevy_logo_dark.png");
    embedded_asset!(app, "branding/birdoggo.png");

    app.run();
}

fn camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
        ..default()
    });
    commands.spawn((
        DirectionalLightBundle {
            transform: Transform::IDENTITY.looking_to(Vec3::new(1.0, -1.0, 1.0), Vec3::Y),
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 0.0,
                ..default()
            },
            ..default()
        },
        DirectionalLightIlluminance(0.0),
    ));
}
