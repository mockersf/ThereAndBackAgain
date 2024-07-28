use std::time::Duration;

use bevy::{color::palettes, prelude::*};
use bevy_easings::{Ease, EaseFunction, EasingType};

use crate::{audio::AudioTrigger, menu::SwitchState, GameState};

const CURRENT_STATE: GameState = GameState::Credits;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(CURRENT_STATE), (spawn_credits,))
            .add_systems(
                Update,
                (button_system, crate::menu::change_state_after_event)
                    .run_if(in_state(CURRENT_STATE)),
            );
    }
}

fn spawn_credits(mut commands: Commands) {
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
                            "Credits",
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
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "made with Bevy Engine",
                            TextStyle {
                                font_size: 30.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        ..default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "3D assets from KayKit",
                            TextStyle {
                                font_size: 30.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        ..default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Audio loops from ZapSplat",
                            TextStyle {
                                font_size: 30.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        ..default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "ui animations with bevy_easings",
                            TextStyle {
                                font_size: 30.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        ..default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "particles with bevy_firework",
                            TextStyle {
                                font_size: 30.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        ..default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "physics with avian3d",
                            TextStyle {
                                font_size: 30.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        ..default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "pathfinding with vleue_navigator",
                            TextStyle {
                                font_size: 30.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        ..default()
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

fn button_system(
    mut commands: Commands,
    interaction_query: Query<(Ref<Interaction>, &BackgroundColor, Entity), Changed<Interaction>>,
    mut next_state: EventWriter<SwitchState>,
    ui_items: Query<(Entity, &MenuItem)>,
    mut audio_trigger: EventWriter<AudioTrigger>,
) {
    for (interaction, color, entity) in &interaction_query {
        if interaction.is_added() {
            continue;
        }
        match *interaction {
            Interaction::Pressed => {
                audio_trigger.send(AudioTrigger::Click);

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
            Interaction::Hovered => {
                commands.entity(entity).insert(color.ease_to(
                    BUTTON_HOVERED,
                    EaseFunction::QuadraticInOut,
                    EasingType::Once {
                        duration: Duration::from_secs_f32(0.25),
                    },
                ));
            }
            Interaction::None => {
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
