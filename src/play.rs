use std::{f32::consts::PI, time::Duration};

use avian3d::{collision::Collider, prelude::RigidBody};
use bevy::{color::palettes, prelude::*};
use bevy_easings::{Ease, EaseFunction, EaseMethod, EasingType};
use rand::Rng;

use crate::{
    assets::GameAssets,
    game::{ActiveLevel, GameEvent, NavMesh},
    levels::{spawn_level, Bonus, Level, Tile},
    menu::SwitchState,
    GameProgress, GameState,
};

const CURRENT_STATE: GameState = GameState::InGame;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(CURRENT_STATE), (spawn_message,))
            .add_systems(
                Update,
                (
                    button_system,
                    update_progress,
                    display_and_check_conditions,
                    draw_cursor,
                    update_navmesh,
                    #[cfg(feature = "debug")]
                    crate::menu::display_navmesh,
                )
                    .run_if(in_state(CURRENT_STATE)),
            )
            .add_systems(
                PreUpdate,
                change_state_after_event.run_if(in_state(CURRENT_STATE)),
            );
    }
}

#[derive(Resource, Default)]
pub struct GameInProgress {
    pub level: usize,
    pub score: u32,
    pub lost_hobbits: u32,
    pub bonus: Vec<Bonus>,
}

fn spawn_message(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut game: ResMut<GameInProgress>,
    levels: Res<Assets<Level>>,
    mut camera_position: Query<(Entity, &mut Transform), With<Camera>>,
) {
    info!("Loading screen");

    let level: &Level = levels.get(&assets.levels[game.level]).unwrap();
    game.bonus.clone_from(&level.bonus);

    let (level_size, mesh) = spawn_level(
        &mut commands,
        level,
        assets.as_ref(),
        StateScoped(CURRENT_STATE),
    );
    let camera_distance = (level_size.0 as f32 * 1.8).max(level_size.1 as f32);
    let (entity, mut transform) = camera_position.single_mut();
    if level.message.is_some() {
        *transform = Transform::from_translation(Vec3::new(
            level_size.1 as f32 / 2.0,
            4000.0,
            level_size.0 as f32 * 3.0 / 4.0,
        ))
        .looking_at(
            Vec3::new(level_size.1 as f32 / 2.0, 0.0, level_size.0 as f32 / 2.0),
            Vec3::Y,
        );
        commands.entity(entity).insert(
            transform
                .ease_to(
                    Transform::from_translation(Vec3::new(
                        level_size.1 as f32 / 2.0,
                        camera_distance,
                        level_size.0 as f32 * 1.2,
                    ))
                    .looking_at(
                        Vec3::new(level_size.1 as f32 / 2.0, 0.0, level_size.0 as f32 / 4.0),
                        Vec3::Y,
                    ),
                    EaseFunction::QuadraticInOut,
                    EasingType::Once {
                        duration: Duration::from_secs_f32(8.0),
                    },
                )
                .delay(Duration::from_secs_f32(2.0)),
        );
    } else {
        *transform = Transform::from_translation(Vec3::new(
            level_size.1 as f32 / 2.0,
            1000.0,
            level_size.0 as f32 * 3.0 / 4.0,
        ))
        .looking_at(
            Vec3::new(level_size.1 as f32 / 2.0, 0.0, level_size.0 as f32 / 2.0),
            Vec3::Y,
        );
        commands.entity(entity).insert(
            transform.ease_to(
                Transform::from_translation(Vec3::new(
                    level_size.1 as f32 / 2.0,
                    camera_distance,
                    level_size.0 as f32 * 1.2,
                ))
                .looking_at(
                    Vec3::new(level_size.1 as f32 / 2.0, 0.0, level_size.0 as f32 / 4.0),
                    Vec3::Y,
                ),
                EaseFunction::QuadraticInOut,
                EasingType::Once {
                    duration: Duration::from_secs_f32(4.0),
                },
            ),
        );
    }

    commands.insert_resource(ActiveLevel(level.clone()));
    commands.insert_resource(NavMesh(mesh));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                ..default()
            },
            MenuItem::Root,
            StateScoped(CURRENT_STATE),
        ))
        .with_children(|parent| {
            let message_panel_style = Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Percent(5.0)),
                width: Val::Percent(40.0),
                height: Val::Percent(60.0),
                position_type: PositionType::Absolute,
                top: Val::Percent(-100.0),
                left: Val::Percent(30.0),
                ..default()
            };
            parent
                .spawn((
                    NodeBundle {
                        background_color: palettes::tailwind::GREEN_400.into(),
                        border_radius: BorderRadius::all(Val::Percent(5.0)),
                        z_index: ZIndex::Global(1),
                        style: message_panel_style.clone(),
                        ..default()
                    },
                    if level.message.is_some() {
                        message_panel_style
                            .clone()
                            .ease_to(
                                Style {
                                    top: Val::Percent(20.0),
                                    left: Val::Percent(30.0),
                                    ..message_panel_style.clone()
                                },
                                EaseFunction::QuadraticOut,
                                EasingType::Once {
                                    duration: Duration::from_secs_f32(1.0),
                                },
                            )
                            .ease_to(
                                Style {
                                    width: Val::Percent(70.0),
                                    height: Val::Percent(20.0),
                                    top: Val::Percent(0.0),
                                    left: Val::Percent(30.0),
                                    ..message_panel_style.clone()
                                },
                                EaseFunction::QuadraticOut,
                                EasingType::Once {
                                    duration: Duration::from_secs_f32(1.0),
                                },
                            )
                            .delay(Duration::from_secs_f32(6.0))
                    } else {
                        Style {
                            width: Val::Percent(30.0),
                            height: Val::Percent(3.0),
                            left: Val::Percent(70.0),
                            ..message_panel_style.clone()
                        }
                        .ease_to(
                            Style {
                                top: Val::Percent(0.0),
                                left: Val::Percent(70.0),
                                width: Val::Percent(30.0),
                                height: Val::Percent(3.0),
                                ..message_panel_style.clone()
                            },
                            EaseFunction::QuadraticOut,
                            EasingType::Once {
                                duration: Duration::from_secs_f32(1.0),
                            },
                        )
                        .ease_to(
                            Style {
                                width: Val::Percent(30.0),
                                height: Val::Percent(3.0),
                                top: Val::Percent(0.0),
                                left: Val::Percent(70.0),
                                ..message_panel_style.clone()
                            },
                            EaseFunction::QuadraticOut,
                            EasingType::Once {
                                duration: Duration::from_secs_f32(1.0),
                            },
                        )
                    },
                    MenuItem::Panel,
                ))
                .with_children(|parent| {
                    if let Some(message) = level.message.as_ref() {
                        parent.spawn(TextBundle {
                            text: Text::from_section(
                                message.clone(),
                                TextStyle {
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ),
                            ..default()
                        });

                        parent.spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(5.0),
                                    ..default()
                                },
                                background_color: palettes::tailwind::INDIGO_800.into(),
                                ..default()
                            },
                            Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(5.0),
                                ..default()
                            }
                            .ease_to(
                                Style {
                                    width: Val::Percent(0.0),
                                    height: Val::Px(5.0),
                                    ..default()
                                },
                                EaseMethod::Linear,
                                EasingType::Once {
                                    duration: Duration::from_secs_f32(6.0),
                                },
                            ),
                        ));
                    }

                    let button_style = Style {
                        width: Val::Px(150.0),
                        height: Val::Px(30.0),
                        border: UiRect::all(Val::Px(3.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::top(Val::Px(20.0)),
                        justify_self: JustifySelf::End,
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(10.0),
                        right: Val::Px(10.0),
                        ..default()
                    };

                    parent
                        .spawn((
                            ButtonBundle {
                                background_color: palettes::tailwind::INDIGO_800.into(),
                                border_radius: BorderRadius::all(Val::Percent(10.0)),
                                border_color: BorderColor(palettes::tailwind::INDIGO_400.into()),
                                style: button_style.clone(),
                                ..default()
                            },
                            button_style.clone().ease_to(
                                Style {
                                    border: UiRect::all(Val::Px(5.0)),
                                    ..button_style.clone()
                                },
                                EaseFunction::QuadraticInOut,
                                EasingType::PingPong {
                                    duration: Duration::from_secs_f32(1.0),
                                    pause: Some(Duration::from_secs_f32(0.5)),
                                },
                            ),
                            MenuItem::Button,
                            ButtonAction::Back,
                        ))
                        .with_children(|p| {
                            p.spawn(TextBundle {
                                text: Text::from_section(
                                    "Back to Menu",
                                    TextStyle {
                                        font_size: 18.0,
                                        ..default()
                                    },
                                ),
                                ..default()
                            });
                        });
                });

            let progress_panel_style = Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Percent(5.0)),
                width: Val::Percent(20.0),
                height: Val::Percent(40.0),
                position_type: PositionType::Absolute,
                top: Val::Percent(0.0),
                left: Val::Percent(-50.0),
                ..default()
            };

            {
                parent
                    .spawn((
                        NodeBundle {
                            background_color: palettes::tailwind::GREEN_400.into(),
                            border_radius: BorderRadius::all(Val::Percent(5.0)),
                            z_index: ZIndex::Global(1),
                            style: progress_panel_style.clone(),
                            ..default()
                        },
                        progress_panel_style
                            .clone()
                            .ease_to(
                                Style {
                                    left: Val::Percent(0.0),
                                    ..progress_panel_style.clone()
                                },
                                EaseFunction::QuadraticOut,
                                EasingType::Once {
                                    duration: Duration::from_secs_f32(1.0),
                                },
                            )
                            .delay(Duration::from_secs_f32(if level.message.is_some() {
                                6.0
                            } else {
                                0.0
                            })),
                        MenuItem::Panel,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TextBundle {
                                text: Text::from_sections([
                                    TextSection {
                                        value: "Treasures: ".to_string(),
                                        style: TextStyle {
                                            font_size: 20.0,
                                            color: Color::WHITE,
                                            ..default()
                                        },
                                    },
                                    TextSection {
                                        value: "0".to_string(),
                                        style: TextStyle {
                                            font_size: 20.0,
                                            color: Color::WHITE,
                                            ..default()
                                        },
                                    },
                                    TextSection {
                                        value: format!(" / {}", level.treasures),
                                        style: TextStyle {
                                            font_size: 20.0,
                                            color: Color::WHITE,
                                            ..default()
                                        },
                                    },
                                ]),
                                ..default()
                            },
                            StatusText::Treasures,
                        ));
                        if let Some(max_lost) = level.losts {
                            parent.spawn((
                                TextBundle {
                                    text: Text::from_sections([
                                        TextSection {
                                            value: "Lost Hobbits: ".to_string(),
                                            style: TextStyle {
                                                font_size: 20.0,
                                                color: Color::WHITE,
                                                ..default()
                                            },
                                        },
                                        TextSection {
                                            value: "0".to_string(),
                                            style: TextStyle {
                                                font_size: 20.0,
                                                color: Color::WHITE,
                                                ..default()
                                            },
                                        },
                                        TextSection {
                                            value: format!(" / {}", max_lost),
                                            style: TextStyle {
                                                font_size: 20.0,
                                                color: Color::WHITE,
                                                ..default()
                                            },
                                        },
                                    ]),
                                    ..default()
                                },
                                StatusText::HobbitsLost,
                            ));
                        }
                        if let Some(goal) = &level.goal {
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    goal.clone(),
                                    TextStyle {
                                        font_size: 20.0,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                ),
                                ..default()
                            });
                        }
                        parent
                            .spawn((
                                NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::Row,
                                        flex_wrap: FlexWrap::Wrap,
                                        ..default()
                                    },
                                    ..default()
                                },
                                MenuItem::BonusPanel,
                            ))
                            .with_children(|parent| {
                                for bonus in &game.bonus {
                                    let button_style = Style {
                                        width: Val::Px(50.0),
                                        height: Val::Px(50.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        margin: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    };

                                    parent
                                        .spawn((
                                            ButtonBundle {
                                                background_color: palettes::tailwind::INDIGO_800
                                                    .into(),
                                                border_radius: BorderRadius::all(Val::Percent(
                                                    10.0,
                                                )),
                                                border_color: BorderColor(
                                                    palettes::tailwind::INDIGO_400.into(),
                                                ),
                                                style: button_style.clone(),
                                                ..default()
                                            },
                                            button_style.clone().ease_to(
                                                Style {
                                                    border: UiRect::all(Val::Px(6.0)),
                                                    ..button_style.clone()
                                                },
                                                EaseFunction::QuadraticInOut,
                                                EasingType::PingPong {
                                                    duration: Duration::from_secs_f32(0.5),
                                                    pause: None,
                                                },
                                            ),
                                            MenuItem::Button,
                                            ButtonAction::Bonus(*bonus),
                                        ))
                                        .with_children(|p| {
                                            p.spawn(ImageBundle {
                                                image: UiImage::new(match bonus {
                                                    Bonus::Obstacle => assets.icon_obstacle.clone(),
                                                }),
                                                style: Style {
                                                    width: Val::Px(40.0),
                                                    height: Val::Px(40.0),
                                                    ..default()
                                                },

                                                ..default()
                                            });
                                        });
                                }
                            });
                    });
            }
        });
}

#[derive(Component, Default, Clone)]
struct ImageColor {
    color: Srgba,
}

impl bevy_easings::Lerp for ImageColor {
    type Scalar = f32;

    fn lerp(&self, other: &Self, scalar: &Self::Scalar) -> Self {
        ImageColor {
            color: self.color.mix(&other.color, *scalar),
        }
    }
}

#[derive(Component, PartialEq, Eq)]
enum MenuItem {
    Root,
    Panel,
    Button,
    BonusPanel,
}

#[derive(Component, PartialEq, Eq)]
enum ButtonAction {
    Back,
    Bonus(Bonus),
    RemoveBonus(Bonus, Entity),
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut commands: Commands,
    interaction_query: Query<(
        Ref<Interaction>,
        &BackgroundColor,
        Entity,
        &ButtonAction,
        Option<&SelectedBonus>,
    )>,
    mut next_state: EventWriter<SwitchState>,
    ui_items: Query<(Entity, &MenuItem, &Style)>,
    camera_position: Query<(Entity, &Transform), With<Camera>>,
    assets: Res<GameAssets>,
) {
    for (interaction, color, entity, action, selected) in &interaction_query {
        if !interaction.is_changed() {
            continue;
        }
        if interaction.is_added() {
            continue;
        }
        match *interaction {
            Interaction::Pressed => match action {
                ButtonAction::Back => {
                    next_state.send(SwitchState(GameState::Menu));

                    let (entity, transform) = camera_position.single();
                    commands.entity(entity).insert(transform.ease_to(
                        Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
                        EaseFunction::QuadraticInOut,
                        EasingType::Once {
                            duration: Duration::from_secs_f32(1.0),
                        },
                    ));

                    for (entity, kind, style) in &ui_items {
                        if *kind == MenuItem::Panel {
                            commands.entity(entity).insert(style.clone().ease_to(
                                Style {
                                    top: Val::Percent(-50.0),
                                    ..style.clone()
                                },
                                EaseFunction::QuadraticOut,
                                EasingType::Once {
                                    duration: Duration::from_secs_f32(1.0),
                                },
                            ));
                        }
                    }

                    commands.entity(entity).insert(color.ease_to(
                        BUTTON_HOVERED,
                        EaseFunction::QuadraticInOut,
                        EasingType::Once {
                            duration: Duration::from_secs_f32(0.25),
                        },
                    ));
                }
                ButtonAction::Bonus(_) => {
                    if selected.is_none() {
                        commands.entity(entity).insert((
                            color.ease_to(
                                BUTTON_SELECTED,
                                EaseFunction::QuadraticInOut,
                                EasingType::Once {
                                    duration: Duration::from_secs_f32(0.25),
                                },
                            ),
                            SelectedBonus,
                        ));
                        for (_, _, entity, _, selected) in &interaction_query {
                            if selected.is_some() {
                                commands
                                    .entity(entity)
                                    .insert(color.ease_to(
                                        BUTTON_IDLE,
                                        EaseFunction::QuadraticInOut,
                                        EasingType::Once {
                                            duration: Duration::from_secs_f32(0.25),
                                        },
                                    ))
                                    .remove::<SelectedBonus>();
                            }
                        }
                    } else {
                        commands
                            .entity(entity)
                            .insert(color.ease_to(
                                BUTTON_HOVERED,
                                EaseFunction::QuadraticInOut,
                                EasingType::Once {
                                    duration: Duration::from_secs_f32(0.25),
                                },
                            ))
                            .remove::<SelectedBonus>();
                    }
                }
                ButtonAction::RemoveBonus(original_bonus, to_remove) => {
                    commands.entity(*to_remove).despawn_recursive();
                    commands
                        .entity(entity)
                        .despawn_descendants()
                        .insert(ButtonAction::Bonus(*original_bonus))
                        .with_children(|p| {
                            p.spawn(ImageBundle {
                                image: UiImage::new(match original_bonus {
                                    Bonus::Obstacle => assets.icon_obstacle.clone(),
                                }),
                                style: Style {
                                    width: Val::Px(40.0),
                                    height: Val::Px(40.0),
                                    ..default()
                                },

                                ..default()
                            });
                        });
                }
            },
            Interaction::Hovered => {
                if selected.is_some() {
                    continue;
                }
                commands.entity(entity).insert(color.ease_to(
                    BUTTON_HOVERED,
                    EaseFunction::QuadraticInOut,
                    EasingType::Once {
                        duration: Duration::from_secs_f32(0.25),
                    },
                ));
            }
            Interaction::None => {
                if selected.is_some() {
                    continue;
                }
                if matches!(action, ButtonAction::RemoveBonus(_, _)) {
                    commands.entity(entity).insert(color.ease_to(
                        BUTTON_IDLE_REMOVE,
                        EaseFunction::QuadraticInOut,
                        EasingType::Once {
                            duration: Duration::from_secs_f32(0.25),
                        },
                    ));
                } else {
                    commands.entity(entity).insert(color.ease_to(
                        BUTTON_IDLE,
                        EaseFunction::QuadraticInOut,
                        EasingType::Once {
                            duration: Duration::from_secs_f32(0.25),
                        },
                    ));
                }
            }
        }
    }
}

#[derive(Component)]
struct SelectedBonus;

const BUTTON_IDLE: BackgroundColor = BackgroundColor(Color::Srgba(palettes::tailwind::INDIGO_800));
const BUTTON_IDLE_REMOVE: BackgroundColor =
    BackgroundColor(Color::Srgba(palettes::tailwind::GRAY_600));
const BUTTON_HOVERED: BackgroundColor =
    BackgroundColor(Color::Srgba(palettes::tailwind::AMBER_600));
const BUTTON_SELECTED: BackgroundColor = BackgroundColor(Color::Srgba(palettes::tailwind::SKY_300));

pub fn change_state_after_event(
    mut commands: Commands,
    mut event_reader: EventReader<SwitchState>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut triggered: Local<Option<(Timer, GameState)>>,
) {
    if let Some((timer, next)) = triggered.as_mut() {
        if timer.tick(time.delta()).just_finished() {
            next_state.set(*next);
            *triggered = None;
            commands.remove_resource::<ActiveLevel>();
        }
    } else if let Some(next) = event_reader.read().last() {
        *triggered = Some((Timer::from_seconds(1.0, TimerMode::Once), next.0));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
enum StatusText {
    Treasures,
    HobbitsLost,
}

fn update_progress(mut game_events: EventReader<GameEvent>, mut game: ResMut<GameInProgress>) {
    for event in game_events.read() {
        match event {
            GameEvent::HomeWithTreasure => {
                game.score += 1;
            }
            GameEvent::CollidedWithHobbit => {
                game.lost_hobbits += 1;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn display_and_check_conditions(
    game: Res<GameInProgress>,
    mut progress: ResMut<GameProgress>,
    mut commands: Commands,
    mut next_state: EventWriter<SwitchState>,
    ui_items: Query<(Entity, &MenuItem, &Style)>,
    camera_position: Query<(Entity, &Transform), With<Camera>>,
    assets: Res<GameAssets>,
    levels: Res<Assets<Level>>,
    mut texts: Query<(&mut Text, &StatusText)>,
) {
    if game.is_changed() {
        for (mut text, kind) in &mut texts {
            match kind {
                StatusText::Treasures => {
                    text.sections[1].value = game.score.to_string();
                }
                StatusText::HobbitsLost => {
                    text.sections[1].value = game.lost_hobbits.to_string();
                }
            }
        }

        let level = levels.get(&assets.levels[game.level]).unwrap();
        if game.score == level.treasures {
            progress.current_level = game.level + 1;
            next_state.send(SwitchState(GameState::Win));

            let (entity, transform) = camera_position.single();
            commands.entity(entity).insert(transform.ease_to(
                Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
                EaseFunction::QuadraticInOut,
                EasingType::Once {
                    duration: Duration::from_secs_f32(1.0),
                },
            ));

            for (entity, kind, style) in &ui_items {
                if *kind == MenuItem::Panel {
                    commands.entity(entity).insert(style.clone().ease_to(
                        Style {
                            top: Val::Percent(-50.0),
                            ..style.clone()
                        },
                        EaseFunction::QuadraticOut,
                        EasingType::Once {
                            duration: Duration::from_secs_f32(1.0),
                        },
                    ));
                }
            }
        }
        if Some(game.lost_hobbits) == level.losts {
            next_state.send(SwitchState(GameState::Lost));

            let (entity, transform) = camera_position.single();
            commands.entity(entity).insert(transform.ease_to(
                Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
                EaseFunction::QuadraticInOut,
                EasingType::Once {
                    duration: Duration::from_secs_f32(1.0),
                },
            ));

            for (entity, kind, style) in &ui_items {
                if *kind == MenuItem::Panel {
                    commands.entity(entity).insert(style.clone().ease_to(
                        Style {
                            top: Val::Percent(-50.0),
                            ..style.clone()
                        },
                        EaseFunction::QuadraticOut,
                        EasingType::Once {
                            duration: Duration::from_secs_f32(1.0),
                        },
                    ));
                }
            }
        }
    }
}

#[derive(Component)]
struct SpawnedObstacle;

#[allow(clippy::too_many_arguments)]
fn draw_cursor(
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
    game: Res<GameInProgress>,
    assets: Res<GameAssets>,
    levels: Res<Assets<Level>>,
    selected: Query<(Entity, &ButtonAction), With<SelectedBonus>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    obstacles: Query<&Transform, With<SpawnedObstacle>>,
    mut navmesh: ResMut<NavMesh>,
) {
    if let Ok((entity, button)) = selected.get_single() {
        let (camera, camera_transform) = camera_query.single();
        let ground = GlobalTransform::default();

        let level = levels.get(&assets.levels[game.level]).unwrap();

        let Some(cursor_position) = windows.single().cursor_position() else {
            return;
        };

        // Calculate a ray pointing from the camera into the world based on the cursor's position.
        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };

        // Calculate if and where the ray is hitting the ground plane.
        let Some(distance) =
            ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
        else {
            return;
        };
        let point = ray.get_point(distance);
        let normalized_point = Vec3::new((point.x / 4.0).round(), 0.1, (point.z / 4.0).round());

        let existing_obstacles = obstacles
            .iter()
            .map(|t| (t.translation.x as usize / 4, t.translation.z as usize / 4))
            .collect::<Vec<_>>();

        if Some(&Tile::Floor)
            == level.floors[0]
                .get(if normalized_point.z < 0.0 {
                    usize::MAX
                } else {
                    normalized_point.z as usize
                })
                .and_then(|r| {
                    r.get(if normalized_point.x < 0.0 {
                        usize::MAX
                    } else {
                        normalized_point.x as usize
                    })
                })
        {
            if let ButtonAction::Bonus(bonus_to_add) = button {
                if existing_obstacles
                    .contains(&(normalized_point.x as usize, normalized_point.z as usize))
                {
                    return;
                }
                gizmos.circle(
                    normalized_point * 4.0,
                    ground.up(),
                    1.3,
                    palettes::tailwind::GREEN_400,
                );
                gizmos.circle(
                    normalized_point * 4.0,
                    ground.up(),
                    1.2,
                    palettes::tailwind::GREEN_500,
                );
                gizmos.circle(
                    normalized_point * 4.0,
                    ground.up(),
                    1.1,
                    palettes::tailwind::GREEN_600,
                );
                if mouse_input.just_pressed(MouseButton::Left) {
                    let obstacle_entity = commands
                        .spawn((
                            SceneBundle {
                                scene: match bonus_to_add {
                                    Bonus::Obstacle => assets.obstacle.clone(),
                                },
                                transform: Transform::from_translation(normalized_point * 4.0)
                                    .with_rotation(Quat::from_rotation_y(
                                        rand::thread_rng().gen_range(0.0..(2.0 * PI)),
                                    ))
                                    .with_scale(Vec3::splat(1.5)),
                                ..default()
                            },
                            SpawnedObstacle,
                            RigidBody::Static,
                            Collider::cylinder(1.0, 2.0),
                            StateScoped(CURRENT_STATE),
                        ))
                        .id();
                    commands
                        .entity(entity)
                        .insert((
                            BUTTON_IDLE_REMOVE,
                            ButtonAction::RemoveBonus(*bonus_to_add, obstacle_entity),
                        ))
                        .remove::<SelectedBonus>()
                        .with_children(|p| {
                            p.spawn(TextBundle {
                                text: Text::from_section(
                                    "X",
                                    TextStyle {
                                        font_size: 30.0,
                                        color: palettes::tailwind::RED_600.into(),
                                        ..default()
                                    },
                                ),
                                style: Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    position_type: PositionType::Absolute,
                                    justify_content: JustifyContent::Center,
                                    align_self: AlignSelf::Center,
                                    ..default()
                                },
                                background_color: BackgroundColor(
                                    palettes::tailwind::GRAY_400.with_alpha(0.5).into(),
                                ),
                                ..default()
                            });
                        });
                    navmesh.0 = level.as_navmesh(
                        obstacles
                            .iter()
                            .map(|t| (t.translation.x as usize / 4, t.translation.z as usize / 4))
                            .chain(std::iter::once((
                                normalized_point.x as usize,
                                normalized_point.z as usize,
                            )))
                            .collect(),
                    );
                }
            }
        }
    }
}

fn update_navmesh(
    game: Res<GameInProgress>,
    assets: Res<GameAssets>,
    levels: Res<Assets<Level>>,
    obstacles: Query<&Transform, With<SpawnedObstacle>>,
    mut navmesh: ResMut<NavMesh>,
    removed_obstacles: RemovedComponents<SpawnedObstacle>,
) {
    if !removed_obstacles.is_empty() {
        let level = levels.get(&assets.levels[game.level]).unwrap();
        navmesh.0 = level.as_navmesh(
            obstacles
                .iter()
                .map(|t| (t.translation.x as usize / 4, t.translation.z as usize / 4))
                .collect(),
        );
    }
}
