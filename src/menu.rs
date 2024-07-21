use std::time::Duration;

use bevy::{color::palettes, prelude::*};
use bevy_easings::{Ease, EaseFunction, EasingType};

use crate::GameState;

const CURRENT_STATE: GameState = GameState::Menu;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(CURRENT_STATE), spawn_menu)
            .add_systems(Update, update_text.run_if(in_state(CURRENT_STATE)));
    }
}

fn spawn_menu(mut commands: Commands) {
    info!("Loading screen");

    commands
        .spawn((
            NodeBundle { ..default() },
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
                    left: Val::Percent(0.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                EaseFunction::QuadraticOut,
                EasingType::Once {
                    duration: Duration::from_secs(1),
                },
            ),
            MenuItem::Root,
            StateScoped(CURRENT_STATE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        background_color: palettes::tailwind::EMERALD_400.into(),
                        border_radius: BorderRadius::right(Val::Percent(5.0)),
                        border_color: BorderColor(palettes::tailwind::EMERALD_100.into()),
                        z_index: ZIndex::Global(1),
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            width: Val::Px(300.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        ..default()
                    },
                    MenuItem::Panel,
                ))
                .with_children(|parent| {
                    for i in 0..2 {
                        parent
                            .spawn((
                                ButtonBundle {
                                    background_color: palettes::tailwind::INDIGO_800.into(),
                                    border_radius: BorderRadius::all(Val::Percent(10.0)),
                                    border_color: BorderColor(
                                        palettes::tailwind::INDIGO_400.into(),
                                    ),
                                    style: Style {
                                        width: Val::Px(0.0),
                                        height: Val::Px(0.0),
                                        top: Val::Px(30.0 + i as f32 * 70.0),
                                        border: UiRect::all(Val::Px(0.0)),
                                        position_type: PositionType::Absolute,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    ..default()
                                },
                                Style {
                                    width: Val::Px(0.0),
                                    height: Val::Px(0.0),
                                    top: Val::Px(30.0 + i as f32 * 70.0),
                                    border: UiRect::all(Val::Px(0.0)),
                                    position_type: PositionType::Absolute,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                }
                                .ease_to(
                                    Style {
                                        width: Val::Px(250.0),
                                        height: Val::Px(65.0),
                                        border: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                    EaseFunction::BounceOut,
                                    EasingType::Once {
                                        duration: Duration::from_secs_f32(1.2),
                                    },
                                )
                                .delay(Duration::from_secs_f32(0.2 * i as f32)),
                                MenuItem::Button(i),
                            ))
                            .with_children(|p| {
                                p.spawn(TextBundle {
                                    text: Text::from_section(
                                        match i {
                                            0 => "Level Select",
                                            1 => "Quit",
                                            _ => unreachable!(),
                                        },
                                        TextStyle {
                                            font_size: 0.0,
                                            ..default()
                                        },
                                    ),
                                    ..default()
                                });
                            });
                    }
                });
        });
}

#[derive(Component)]
enum MenuItem {
    Root,
    Panel,
    Button(u32),
}

fn update_text(mut text: Query<(&mut Text, &Parent)>, nodes: Query<&Node>) {
    for (mut text, parent) in text.iter_mut() {
        let node = nodes.get(parent.get()).unwrap();
        text.sections[0].style.font_size = (node.size().y / 4.0).floor() * 2.0;
    }
}
