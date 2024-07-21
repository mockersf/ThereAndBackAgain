use bevy::{
    asset::{embedded_asset, AssetMetaCheck},
    prelude::*,
};

mod assets;
mod loading;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    Menu,
    SetupLevel(usize),
    RunLevel(usize),
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "There And Back Again".to_string(),
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
    .add_plugins((loading::Plugin,))
    .add_systems(Startup, camera);

    embedded_asset!(app, "branding/logo.png");
    embedded_asset!(app, "branding/bevy_logo_dark.png");
    embedded_asset!(app, "branding/birdoggo.png");

    app.run();
}

fn camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle { ..default() });
}
