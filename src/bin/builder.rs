#![windows_subsystem = "windows"]

use avian3d::prelude::*;
#[cfg(feature = "debug")]
use bevy::window::PresentMode;
use bevy::{asset::AssetMetaCheck, core_pipeline::bloom::BloomSettings, prelude::*};
use bevy_easings::EasingsPlugin;
use bevy_firework::plugin::ParticleSystemPlugin;

use there_and_back_again::{
    credits, game, level_selector,
    levels::{self, Bonus, Level},
    loading, lost, menu,
    play::{self, GameInProgress},
    win, GameProgress, GameState,
};

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
        level_selector::Plugin,
        play::Plugin,
        win::Plugin,
        lost::Plugin,
        ReloadPlugin,
    ))
    .add_systems(Startup, camera);

    app.insert_resource(GameProgress {
        current_level: if cfg!(feature = "debug") {
            usize::MAX
        } else {
            1
        },
    });

    #[cfg(feature = "debug")]
    app.add_plugins(PhysicsDebugPlugin::default());

    app.insert_resource(GameInProgress {
        level: 0,
        score: 0,
        lost_hobbits: 0,
        bonus: vec![
            Bonus::Obstacle,
            Bonus::Obstacle,
            Bonus::Obstacle,
            Bonus::Obstacle,
        ],
    });

    app.run();
}

fn camera(mut commands: Commands) {
    commands.spawn((
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

struct ReloadPlugin;
impl bevy::app::Plugin for ReloadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Reload),
            |mut next_state: ResMut<NextState<GameState>>| {
                next_state.set(GameState::InGame);
            },
        )
        .add_systems(
            Update,
            (|mut next_state: ResMut<NextState<GameState>>,
              asset_event: EventReader<AssetEvent<Level>>| {
                if !asset_event.is_empty() {
                    next_state.set(GameState::Reload);
                }
            })
            .run_if(in_state(GameState::InGame)),
        );
    }
}
