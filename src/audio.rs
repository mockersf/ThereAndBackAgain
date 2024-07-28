use bevy::prelude::*;

use crate::GameState;

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AudioTrigger>()
            .add_systems(OnEnter(GameState::Loading), load_background_music)
            .add_systems(OnEnter(GameState::InGame), switch_to_game_music)
            .add_systems(OnExit(GameState::InGame), switch_to_menu_music)
            .add_systems(Update, (fade_in, fade_out, play_audio_effect));
    }
}

#[derive(Resource, Clone)]
struct Soundtracks {
    menu: Handle<AudioSource>,
    game: Handle<AudioSource>,
}

#[derive(Resource, Clone)]
struct AudioEffects {
    click: Handle<AudioSource>,
    home: Handle<AudioSource>,
    hurt: Handle<AudioSource>,
    lost: Handle<AudioSource>,
    obstacle: Handle<AudioSource>,
    spawn: Handle<AudioSource>,
    start: Handle<AudioSource>,
    treasure: Handle<AudioSource>,
    win: Handle<AudioSource>,
}

#[derive(Component)]
struct FadeIn;
#[derive(Component)]
struct FadeOut;

fn load_background_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    let soundtracks = Soundtracks {
        menu: asset_server.load("music_zapsplat_game_music_zen_calm_soft_arpeggios_013.ogg"),
        game: asset_server
            .load("music_zapsplat_game_music_dark_atmospheric_slow_beat_zombie_019.ogg"),
    };
    commands.insert_resource(soundtracks.clone());

    commands.spawn((
        AudioBundle {
            source: soundtracks.menu.clone(),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::ZERO,
                ..default()
            },
        },
        FadeIn,
    ));

    commands.insert_resource(AudioEffects {
        click: asset_server.load("audio/click.ogg"),
        home: asset_server.load("audio/home.ogg"),
        hurt: asset_server.load("audio/hurt.ogg"),
        lost: asset_server.load("audio/lost.ogg"),
        obstacle: asset_server.load("audio/obstacle.ogg"),
        spawn: asset_server.load("audio/spawn.ogg"),
        start: asset_server.load("audio/start.ogg"),
        treasure: asset_server.load("audio/treasure.ogg"),
        win: asset_server.load("audio/win.ogg"),
    });
}

const FADE_TIME: f32 = 2.0;
const MAX_VOLUME: f32 = 0.08;

fn fade_in(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity), With<FadeIn>>,
    time: Res<Time>,
) {
    for (audio, entity) in audio_sink.iter_mut() {
        audio.set_volume(audio.volume() + time.delta_seconds() * MAX_VOLUME / FADE_TIME);
        if audio.volume() >= MAX_VOLUME {
            audio.set_volume(MAX_VOLUME);
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}

fn fade_out(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity), With<FadeOut>>,
    time: Res<Time>,
) {
    for (audio, entity) in audio_sink.iter_mut() {
        audio.set_volume(audio.volume() - time.delta_seconds() * MAX_VOLUME / FADE_TIME);
        if audio.volume() <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn switch_to_game_music(
    mut commands: Commands,
    soundtracks: Res<Soundtracks>,
    mut previous_soundtrack: Query<Entity, With<PlaybackSettings>>,
) {
    for entity in previous_soundtrack.iter_mut() {
        commands.entity(entity).insert(FadeOut);
    }
    commands.spawn((
        AudioBundle {
            source: soundtracks.game.clone(),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::ZERO,
                ..default()
            },
        },
        FadeIn,
    ));
}

fn switch_to_menu_music(
    mut commands: Commands,
    soundtracks: Res<Soundtracks>,
    mut previous_soundtrack: Query<Entity, With<PlaybackSettings>>,
) {
    for entity in previous_soundtrack.iter_mut() {
        commands.entity(entity).insert(FadeOut);
    }
    commands.spawn((
        AudioBundle {
            source: soundtracks.menu.clone(),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::ZERO,
                ..default()
            },
        },
        FadeIn,
    ));
}

#[derive(Event)]
pub enum AudioTrigger {
    Click,
    Home,
    Hurt,
    Lost,
    Obstacle,
    Spawn,
    Start,
    Treasure,
    Win,
}

fn play_audio_effect(
    mut commands: Commands,
    audio_effects: Res<AudioEffects>,
    mut audio_trigger: EventReader<AudioTrigger>,
    state: Res<State<GameState>>,
) {
    for trigger in audio_trigger.read() {
        let handle = match trigger {
            AudioTrigger::Click => audio_effects.click.clone(),
            AudioTrigger::Home => audio_effects.home.clone(),
            AudioTrigger::Hurt => audio_effects.hurt.clone(),
            AudioTrigger::Lost => audio_effects.lost.clone(),
            AudioTrigger::Obstacle => audio_effects.obstacle.clone(),
            AudioTrigger::Spawn => audio_effects.spawn.clone(),
            AudioTrigger::Start => audio_effects.start.clone(),
            AudioTrigger::Treasure => audio_effects.treasure.clone(),
            AudioTrigger::Win => audio_effects.win.clone(),
        };
        commands.spawn(AudioBundle {
            source: handle,
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: bevy::audio::Volume::new(match state.get() {
                    GameState::Menu => 0.1,
                    _ => 0.5,
                }),
                ..default()
            },
        });
    }
}
