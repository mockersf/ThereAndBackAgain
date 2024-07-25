use std::time::Duration;

use bevy::{color::palettes, prelude::*};
use bevy_easings::{Ease, EaseFunction, EasingType};

use crate::{
    assets::GameAssets,
    game::{ActiveLevel, NavMesh},
    levels::{spawn_level, Level},
    menu::SwitchState,
    GameState,
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

#[derive(Resource)]
pub struct GameInProgress {
    pub level: usize,
}

fn spawn_message(
    mut commands: Commands,
    assets: Res<GameAssets>,
    game: Res<GameInProgress>,
    levels: Res<Assets<Level>>,
    mut camera_position: Query<(Entity, &mut Transform), With<Camera>>,
    mut directional_light: Query<(Entity, &mut DirectionalLight)>,
) {
    info!("Loading screen");

    let level = levels.get(&assets.levels[game.level]).unwrap();

    let mut light = directional_light.single_mut();

    let (level_size, mesh) = spawn_level(
        &mut commands,
        level,
        assets.as_ref(),
        StateScoped(CURRENT_STATE),
        (light.0, light.1.as_mut()),
    );
    let (entity, mut transform) = camera_position.single_mut();
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
                    level_size.1 as f32,
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
                        .delay(Duration::from_secs_f32(5.0)),
                    MenuItem::Panel,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            level.message.clone(),
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        ..default()
                    });
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
}

fn button_system(
    mut commands: Commands,
    interaction_query: Query<
        (Ref<Interaction>, &BackgroundColor, Entity, &ButtonAction),
        Changed<Interaction>,
    >,
    mut next_state: EventWriter<SwitchState>,
    ui_items: Query<(Entity, &MenuItem, &Style)>,
    camera_position: Query<(Entity, &Transform), With<Camera>>,
) {
    for (interaction, color, entity, action) in &interaction_query {
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
            },
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
