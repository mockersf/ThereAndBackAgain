use bevy::prelude::*;

pub mod assets;
pub mod credits;
pub mod game;
pub mod level_selector;
pub mod levels;
pub mod loading;
pub mod lost;
pub mod menu;
pub mod play;
pub mod win;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Menu,
    Credits,
    LevelSelect,
    InGame,
    Win,
    Lost,
    Reload,
}

#[derive(Resource)]
pub struct GameProgress {
    pub current_level: usize,
}
