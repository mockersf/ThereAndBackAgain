use std::time::Duration;

use bevy::{color::palettes, prelude::*};
use bevy_easings::{Ease, EaseFunction, EasingType};
use rand::Rng;

use crate::{assets::GameAssets, menu::SwitchState, play::GameInProgress, GameProgress, GameState};

const CURRENT_STATE: GameState = GameState::LevelSelect;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(CURRENT_STATE), (spawn_level_selector,))
            .add_systems(
                Update,
                (button_system, crate::menu::change_state_after_event)
                    .run_if(in_state(CURRENT_STATE)),
            );
    }
}

fn spawn_level_selector(
    mut commands: Commands,
    assets: Res<GameAssets>,
    progress: Res<GameProgress>,
) {
    info!("Loading screen");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    left: Val::Percent(-100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                ..default()
            },
            Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                left: Val::Percent(-100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                ..default()
            }
            .ease_to(
                Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    left: Val::Percent(30.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                EaseFunction::QuadraticOut,
                EasingType::Once {
                    duration: Duration::from_secs_f32(1.0),
                },
            ),
            MenuItem::Root,
            StateScoped(CURRENT_STATE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        background_color: palettes::tailwind::GREEN_400.into(),
                        border_radius: BorderRadius::all(Val::Percent(5.0)),
                        z_index: ZIndex::Global(1),
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            width: Val::Percent(40.0),
                            height: Val::Percent(60.0),
                            ..default()
                        },
                        ..default()
                    },
                    MenuItem::Panel,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Select a Level",
                            TextStyle {
                                font_size: 60.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        style: Style {
                            margin: UiRect::bottom(Val::Percent(5.0)),
                            ..default()
                        },
                        ..default()
                    });

                    let style = Style {
                        width: Val::Px(50.0),
                        height: Val::Px(50.0),
                        border: UiRect::all(Val::Px(3.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        justify_self: JustifySelf::End,
                        ..default()
                    };

                    let style_easing = style.clone().ease_to(
                        Style {
                            width: Val::Px(50.0),
                            height: Val::Px(50.0),
                            border: UiRect::all(Val::Px(5.0)),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_self: JustifySelf::End,

                            ..default()
                        },
                        EaseFunction::QuadraticInOut,
                        EasingType::PingPong {
                            duration: Duration::from_secs_f32(1.0),
                            pause: Some(Duration::from_secs_f32(0.5)),
                        },
                    );
                    let current_easing = style.clone().ease_to(
                        Style {
                            width: Val::Px(50.0),
                            height: Val::Px(50.0),
                            border: UiRect::all(Val::Px(6.0)),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_self: JustifySelf::End,

                            ..default()
                        },
                        EaseFunction::QuadraticInOut,
                        EasingType::PingPong {
                            duration: Duration::from_secs_f32(0.2),
                            pause: Some(Duration::from_secs_f32(0.05)),
                        },
                    );
                    let disabled_easing = style.ease_to(
                        Style {
                            width: Val::Px(50.0),
                            height: Val::Px(50.0),
                            border: UiRect::all(Val::Px(4.0)),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_self: JustifySelf::End,

                            ..default()
                        },
                        EaseFunction::QuadraticInOut,
                        EasingType::PingPong {
                            duration: Duration::from_secs_f32(1.5),
                            pause: Some(Duration::from_secs_f32(0.5)),
                        },
                    );
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_wrap: FlexWrap::Wrap,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            let start = if cfg!(feature = "debug") { 0 } else { 1 };
                            for level in start..assets.levels.len() {
                                parent
                                    .spawn((
                                        ButtonBundle {
                                            background_color: if level <= progress.current_level {
                                                palettes::tailwind::INDIGO_800.into()
                                            } else {
                                                palettes::tailwind::GRAY_400.into()
                                            },
                                            border_radius: BorderRadius::all(Val::Percent(10.0)),
                                            border_color: BorderColor(
                                                palettes::tailwind::INDIGO_400.into(),
                                            ),
                                            style: Style {
                                                width: Val::Px(50.0),
                                                height: Val::Px(50.0),
                                                border: UiRect::all(Val::Px(3.0)),
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                margin: UiRect::all(Val::Px(10.0)),
                                                justify_self: JustifySelf::End,
                                                ..default()
                                            },
                                            ..default()
                                        },
                                        MenuItem::Button,
                                        ButtonAction::Playlevel(level),
                                        match level {
                                            _ if level < progress.current_level => {
                                                style_easing.clone()
                                            }
                                            _ if level == progress.current_level => {
                                                current_easing.clone()
                                            }
                                            _ => disabled_easing.clone(),
                                        }
                                        .delay(
                                            Duration::from_secs_f32(
                                                rand::thread_rng().gen_range(0.0..1.5),
                                            ),
                                        ),
                                    ))
                                    .with_children(|p| {
                                        p.spawn(TextBundle {
                                            text: Text::from_section(
                                                &format!("{}", level),
                                                TextStyle {
                                                    font_size: 20.0,
                                                    ..default()
                                                },
                                            ),
                                            ..default()
                                        });
                                    });
                            }
                        });

                    let button_height = 40.0;
                    let style_easing = Style {
                        width: Val::Px(200.0),
                        height: Val::Px(button_height),
                        border: UiRect::all(Val::Px(3.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::top(Val::Percent(10.0)),
                        justify_self: JustifySelf::End,
                        ..default()
                    };

                    let style_easing = style_easing.ease_to(
                        Style {
                            width: Val::Px(200.0),
                            height: Val::Px(button_height),
                            border: UiRect::all(Val::Px(5.0)),
                            margin: UiRect::top(Val::Percent(10.0)),
                            justify_self: JustifySelf::End,

                            ..default()
                        },
                        EaseFunction::QuadraticInOut,
                        EasingType::PingPong {
                            duration: Duration::from_secs_f32(1.0),
                            pause: Some(Duration::from_secs_f32(0.5)),
                        },
                    );
                    parent
                        .spawn((
                            ButtonBundle {
                                background_color: palettes::tailwind::INDIGO_800.into(),
                                border_radius: BorderRadius::all(Val::Percent(10.0)),
                                border_color: BorderColor(palettes::tailwind::INDIGO_400.into()),
                                style: Style {
                                    width: Val::Px(200.0),
                                    height: Val::Px(button_height),
                                    border: UiRect::all(Val::Px(0.0)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    margin: UiRect::top(Val::Percent(10.0)),
                                    justify_self: JustifySelf::End,
                                    ..default()
                                },
                                ..default()
                            },
                            style_easing,
                            MenuItem::Button,
                            ButtonAction::Back,
                        ))
                        .with_children(|p| {
                            p.spawn(TextBundle {
                                text: Text::from_section(
                                    "Back to Menu",
                                    TextStyle {
                                        font_size: 20.0,
                                        ..default()
                                    },
                                ),
                                ..default()
                            });
                        });
                });
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
}

#[derive(Component, PartialEq, Eq)]
enum ButtonAction {
    Back,
    Playlevel(usize),
}

fn button_system(
    mut commands: Commands,
    interaction_query: Query<
        (Ref<Interaction>, &BackgroundColor, Entity, &ButtonAction),
        Changed<Interaction>,
    >,
    mut next_state: EventWriter<SwitchState>,
    ui_items: Query<(Entity, &MenuItem)>,
    progress: Res<GameProgress>,
) {
    for (interaction, color, entity, action) in &interaction_query {
        if interaction.is_added() {
            continue;
        }
        match *interaction {
            Interaction::Pressed => match action {
                ButtonAction::Back => {
                    next_state.send(SwitchState(GameState::Menu));

                    for (entity, kind) in &ui_items {
                        if *kind == MenuItem::Root {
                            commands.entity(entity).insert(
                                Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    left: Val::Percent(30.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Start,
                                    ..default()
                                }
                                .ease_to(
                                    Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        left: Val::Percent(-100.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Start,
                                        ..default()
                                    },
                                    EaseFunction::QuadraticOut,
                                    EasingType::Once {
                                        duration: Duration::from_secs_f32(1.0),
                                    },
                                ),
                            );
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
                ButtonAction::Playlevel(level) => {
                    if *level <= progress.current_level {
                        next_state.send(SwitchState(GameState::InGame));
                        commands.insert_resource(GameInProgress {
                            level: *level,
                            ..default()
                        });

                        for (entity, kind) in &ui_items {
                            if *kind == MenuItem::Root {
                                commands.entity(entity).insert(
                                    Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        left: Val::Percent(30.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Start,
                                        ..default()
                                    }
                                    .ease_to(
                                        Style {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            left: Val::Percent(-100.0),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Start,
                                            ..default()
                                        },
                                        EaseFunction::QuadraticOut,
                                        EasingType::Once {
                                            duration: Duration::from_secs_f32(1.0),
                                        },
                                    ),
                                );
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
                }
            },
            Interaction::Hovered => {
                if let ButtonAction::Playlevel(level) = action {
                    if progress.current_level < *level {
                        continue;
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
            Interaction::None => {
                if let ButtonAction::Playlevel(level) = action {
                    if progress.current_level < *level {
                        continue;
                    }
                }
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

const BUTTON_IDLE: BackgroundColor = BackgroundColor(Color::Srgba(palettes::tailwind::INDIGO_800));
const BUTTON_HOVERED: BackgroundColor =
    BackgroundColor(Color::Srgba(palettes::tailwind::AMBER_600));
