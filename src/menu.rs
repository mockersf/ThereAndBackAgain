use std::time::Duration;

use bevy::{color::palettes, prelude::*, render::texture::TextureFormatPixelInfo};
use bevy_easings::{CustomComponentEase, Ease, EaseFunction, EasingComponent, EasingType};
use rand::Rng;

use crate::GameState;

const CURRENT_STATE: GameState = GameState::Menu;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(CURRENT_STATE), (spawn_menu,))
            .add_systems(
                Update,
                (
                    update_text,
                    spawn_title_points,
                    animation_maintenance,
                    bevy_easings::custom_ease_system::<ImageColor>,
                )
                    .run_if(in_state(CURRENT_STATE)),
            );
    }
}

fn spawn_menu(mut commands: Commands, window: Query<&Window>) {
    info!("Loading screen");
    let window_size = window.single().size();

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
                    left: Val::Percent(0.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                EaseFunction::QuadraticOut,
                EasingType::Once {
                    duration: Duration::from_secs_f32(1.0),
                },
            )
            .delay(Duration::from_secs_f32(0.5)),
            MenuItem::Root,
            StateScoped(CURRENT_STATE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        background_color: palettes::tailwind::GREEN_400.into(),
                        border_radius: BorderRadius::right(Val::Percent(5.0)),
                        z_index: ZIndex::Global(1),
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
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
                    let nb_buttons = if cfg!(target_arch = "wasm32") { 2 } else { 3 };
                    let button_height = 65.0;
                    for i in 0..nb_buttons {
                        let style_easing = Style {
                            width: Val::Px(0.0),
                            height: Val::Px(0.0),
                            top: Val::Px(
                                window_size.y / nb_buttons as f32 * (i as f32 + 0.5)
                                    - button_height / 2.0,
                            ),
                            border: UiRect::all(Val::Px(0.0)),
                            position_type: PositionType::Absolute,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::bottom(Val::Percent(10.0)),
                            ..default()
                        }
                        .ease_to(
                            Style {
                                width: Val::Px(250.0),
                                height: Val::Px(button_height),
                                border: UiRect::all(Val::Px(3.0)),
                                ..default()
                            },
                            EaseFunction::BounceOut,
                            EasingType::Once {
                                duration: Duration::from_secs_f32(1.2),
                            },
                        )
                        .delay(Duration::from_secs_f32(0.5 + 0.2 * i as f32));

                        let style_easing = if i == 0 {
                            style_easing.ease_to(
                                Style {
                                    width: Val::Px(250.0),
                                    height: Val::Px(button_height),
                                    border: UiRect::all(Val::Px(6.0)),
                                    ..default()
                                },
                                EaseFunction::QuadraticInOut,
                                EasingType::PingPong {
                                    duration: Duration::from_secs_f32(0.2),
                                    pause: Some(Duration::from_secs_f32(0.05)),
                                },
                            )
                        } else {
                            style_easing.ease_to(
                                Style {
                                    width: Val::Px(250.0),
                                    height: Val::Px(button_height),
                                    border: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                                EaseFunction::QuadraticInOut,
                                EasingType::PingPong {
                                    duration: Duration::from_secs_f32(1.0),
                                    pause: Some(Duration::from_secs_f32(0.5)),
                                },
                            )
                        };
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
                                        top: Val::Px(
                                            window_size.y / nb_buttons as f32 * (i as f32 + 0.5)
                                                - button_height / 2.0,
                                        ),
                                        border: UiRect::all(Val::Px(0.0)),
                                        position_type: PositionType::Absolute,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        margin: UiRect::bottom(Val::Percent(10.0)),
                                        ..default()
                                    },
                                    ..default()
                                },
                                style_easing,
                                MenuItem::Button(i),
                            ))
                            .with_children(|p| {
                                p.spawn((
                                    TextBundle {
                                        text: Text::from_section(
                                            match i {
                                                0 => "Level Select",
                                                1 => "Credits",
                                                2 => "Quit",
                                                _ => unreachable!(),
                                            },
                                            TextStyle {
                                                font_size: 0.0,
                                                ..default()
                                            },
                                        ),
                                        ..default()
                                    },
                                    match i {
                                        0 => MenuButton::LevelSelect,
                                        1 => MenuButton::Credits,
                                        2 => MenuButton::Quit,
                                        _ => unreachable!(),
                                    },
                                ));
                            });
                    }
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

fn spawn_title_points(
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    window: Query<&Window>,
    mut commands: Commands,
    mut done: Local<bool>,
    mut png: Local<Option<(Handle<Image>, Handle<Image>)>>,
) {
    if png.is_none() {
        *png = Some((
            asset_server.load("title-1.png"),
            asset_server.load("title-2.png"),
        ));
        return;
    }

    if *done {
        return;
    }
    let Some(image_1) = images.get(&png.as_ref().unwrap().0) else {
        return;
    };
    let Some(image_2) = images.get(&png.as_ref().unwrap().1) else {
        return;
    };

    let resolution: u32 = 6;
    let window_size = window.single().size();

    let point_to_image_duration = Duration::from_secs_f32(0.5);
    let point_placement_duration = Duration::from_secs_f32(1.5);
    let second_image_delay = Duration::from_secs_f32(0.5);

    let title_color = palettes::tailwind::GREEN_400;

    let image_1_position = |i: u32, j: u32| {
        (
            Val::Px(i as f32 / 2.0 + window_size.x / 2.0 - image_1.width() as f32 / 4.0),
            Val::Px(
                j as f32 / 2.0 + window_size.y / 2.0
                    - image_1.height() as f32 / 1.5
                    - window_size.y / 5.0,
            ),
        )
    };
    commands.spawn((
        ImageBundle {
            style: Style {
                width: Val::Px(image_1.width() as f32 / 2.0),
                height: Val::Px(image_1.height() as f32 / 2.0),
                left: image_1_position(0, 0).0,
                top: image_1_position(0, 0).1,
                position_type: PositionType::Absolute,
                ..default()
            },
            image: UiImage::new(png.as_ref().unwrap().0.clone())
                .with_color(title_color.with_alpha(0.0).into()),
            ..default()
        },
        ImageColor {
            color: title_color.with_alpha(0.0).into(),
        }
        .ease_to(
            ImageColor {
                color: title_color.into(),
            },
            EaseFunction::QuadraticInOut,
            EasingType::Once {
                duration: point_to_image_duration,
            },
        )
        .delay(point_placement_duration)
        .with_original_value(),
    ));

    for i in (0..image_1.width()).step_by(resolution as usize) {
        for j in (0..image_1.height()).step_by(resolution as usize) {
            let pixel_size = image_1.texture_descriptor.format.pixel_size();
            let value = image_1
                .data
                .chunks(pixel_size)
                .nth((j * image_1.width() + i) as usize)
                .unwrap();
            // ignore transparent pixels
            if value[3] == 0 {
                continue;
            }
            commands.spawn((
                NodeBundle {
                    z_index: ZIndex::Global(0),
                    border_radius: BorderRadius::MAX,
                    background_color: BackgroundColor(title_color.into()),
                    ..Default::default()
                },
                Style {
                    width: Val::Px(resolution as f32 / 2.0),
                    height: Val::Px(resolution as f32 / 2.0),
                    left: Val::Px(rand::thread_rng().gen_range(0.0..window_size.x)),
                    top: Val::Px(rand::thread_rng().gen_range(0.0..window_size.y)),
                    position_type: PositionType::Absolute,
                    ..Default::default()
                }
                .ease_to(
                    Style {
                        width: Val::Px(resolution as f32 / 2.0),
                        height: Val::Px(resolution as f32 / 2.0),
                        left: image_1_position(i, j).0,
                        top: image_1_position(i, j).1,
                        position_type: PositionType::Absolute,
                        ..Default::default()
                    },
                    EaseFunction::QuadraticInOut,
                    EasingType::Once {
                        duration: point_placement_duration,
                    },
                ),
                BackgroundColor(title_color.into())
                    .ease_to(
                        BackgroundColor(title_color.with_alpha(0.0).into()),
                        EaseFunction::QuadraticInOut,
                        EasingType::Once {
                            duration: point_to_image_duration,
                        },
                    )
                    .delay(point_placement_duration),
                Dot,
            ));
        }
    }

    let image_2_position = |i: u32, j: u32| {
        (
            Val::Px(
                i as f32 / 2.0 + window_size.x / 2.0 - image_2.width() as f32 / 4.0
                    + window_size.x / 5.0,
            ),
            Val::Px(
                j as f32 / 2.0 + window_size.y / 2.0
                    - image_2.height() as f32 / 4.0
                    - window_size.y / 5.0,
            ),
        )
    };

    commands.spawn((
        ImageBundle {
            style: Style {
                width: Val::Px(image_2.width() as f32 / 2.0),
                height: Val::Px(image_2.height() as f32 / 2.0),
                left: image_2_position(0, 0).0,
                top: image_2_position(0, 0).1,
                position_type: PositionType::Absolute,
                ..default()
            },
            image: UiImage::new(png.as_ref().unwrap().1.clone())
                .with_color(title_color.with_alpha(0.0).into()),
            ..default()
        },
        ImageColor {
            color: title_color.with_alpha(0.0).into(),
        }
        .ease_to(
            ImageColor {
                color: title_color.into(),
            },
            EaseFunction::QuadraticInOut,
            EasingType::Once {
                duration: point_to_image_duration,
            },
        )
        .delay(point_placement_duration + second_image_delay)
        .with_original_value(),
    ));

    for i in (0..image_2.width()).step_by(resolution as usize) {
        for j in (0..image_2.height()).step_by(resolution as usize) {
            let pixel_size = image_2.texture_descriptor.format.pixel_size();
            let value = image_2
                .data
                .chunks(pixel_size)
                .nth((j * image_2.width() + i) as usize)
                .unwrap();
            // ignore transparent pixels
            if value[3] == 0 {
                continue;
            }
            let start = Style {
                width: Val::Px(resolution as f32 / 2.0),
                height: Val::Px(resolution as f32 / 2.0),
                left: Val::Px(rand::thread_rng().gen_range(0.0..window_size.x)),
                top: Val::Px(rand::thread_rng().gen_range(0.0..window_size.y)),
                position_type: PositionType::Absolute,
                ..Default::default()
            };
            commands.spawn((
                NodeBundle {
                    z_index: ZIndex::Global(0),
                    border_radius: BorderRadius::MAX,
                    background_color: BackgroundColor(title_color.into()),
                    style: start.clone(),
                    ..Default::default()
                },
                start.ease_to(
                    Style {
                        width: Val::Px(resolution as f32 / 2.0),
                        height: Val::Px(resolution as f32 / 2.0),
                        left: image_2_position(i, j).0,
                        top: image_2_position(i, j).1,
                        position_type: PositionType::Absolute,
                        ..Default::default()
                    },
                    EaseFunction::QuadraticInOut,
                    EasingType::Once {
                        duration: point_placement_duration + second_image_delay,
                    },
                ),
                BackgroundColor(title_color.into())
                    .ease_to(
                        BackgroundColor(title_color.with_alpha(0.0).into()),
                        EaseFunction::QuadraticInOut,
                        EasingType::Once {
                            duration: point_to_image_duration,
                        },
                    )
                    .delay(point_placement_duration + second_image_delay),
                Dot,
            ));
        }
    }

    *done = true;
}

#[derive(Component)]
enum MenuItem {
    Root,
    Panel,
    Button(u32),
}

#[derive(Component)]
struct Dot;

#[derive(Component)]
enum MenuButton {
    LevelSelect,
    Credits,
    Quit,
}

fn update_text(mut text: Query<(&mut Text, &Parent)>, nodes: Query<&Node>) {
    for (mut text, parent) in text.iter_mut() {
        let node = nodes.get(parent.get()).unwrap();
        text.sections[0].style.font_size = (node.size().y / 4.0).floor() * 2.0;
    }
}

fn animation_maintenance(
    mut query: Query<(Ref<ImageColor>, &mut UiImage)>,
    mut removed_transitions: RemovedComponents<EasingComponent<BackgroundColor>>,
    mut commands: Commands,
) {
    // Update color of images
    for (color, mut image) in query.iter_mut() {
        if color.is_changed() {
            image.color = color.color.into();
        }
    }
    // Despawn dots once they are finished
    for entity in removed_transitions.read() {
        commands.entity(entity).despawn();
    }
}
