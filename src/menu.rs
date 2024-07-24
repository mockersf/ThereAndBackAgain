use std::time::Duration;

use bevy::{color::palettes, prelude::*, render::texture::TextureFormatPixelInfo};
use bevy_easings::{CustomComponentEase, Ease, EaseFunction, EasingComponent, EasingType};
use rand::Rng;

use crate::{
    assets::GameAssets,
    game::{ActiveLevel, NavMesh},
    levels::{spawn_level, Level},
    GameProgress, GameState,
};

const CURRENT_STATE: GameState = GameState::Menu;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SwitchState>()
            .add_systems(OnEnter(CURRENT_STATE), (spawn_menu, spawn_level_0))
            .add_systems(
                Update,
                (
                    update_text,
                    spawn_title_points,
                    animation_maintenance,
                    bevy_easings::custom_ease_system::<ImageColor>,
                    button_system,
                    change_state_after_event,
                    spawn_reverse_title_points,
                    #[cfg(feature = "debug")]
                    display_navmesh,
                )
                    .run_if(in_state(CURRENT_STATE)),
            )
            .add_systems(
                PreUpdate,
                change_state_after_event.run_if(in_state(CURRENT_STATE)),
            );
    }
}

fn spawn_level_0(
    mut commands: Commands,
    assets: Res<GameAssets>,
    levels: Res<Assets<Level>>,
    camera_position: Query<(Entity, &Transform), With<Camera>>,
    mut directional_light: Query<(Entity, &mut DirectionalLight)>,
) {
    let mut light = directional_light.single_mut();

    let level = levels.get(&assets.levels[0]).unwrap();
    let (level_size, mesh) = spawn_level(
        &mut commands,
        level,
        assets.as_ref(),
        StateScoped(CURRENT_STATE),
        (light.0, light.1.as_mut()),
    );
    let (entity, transform) = camera_position.single();
    commands.entity(entity).insert(
        transform.ease_to(
            Transform::from_translation(Vec3::new(
                level_size.0 as f32 * 11.0 / 10.0,
                40.0,
                level_size.1 as f32 * 3.0 / 4.0,
            ))
            .looking_at(
                Vec3::new(
                    level_size.0 as f32 * 11.0 / 10.0,
                    0.0,
                    -1.0 * level_size.1 as f32 / 4.0,
                ),
                Vec3::Y,
            ),
            EaseFunction::QuadraticInOut,
            EasingType::Once {
                duration: Duration::from_secs_f32(2.0),
            },
        ),
    );

    commands.insert_resource(ActiveLevel(level.clone()));
    commands.insert_resource(NavMesh(vleue_navigator::NavMesh::from_polyanya_mesh(mesh)));
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
                    let nb_buttons = if cfg!(target_arch = "wasm32") { 3 } else { 4 };
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
                                MenuItem::Button,
                                match i {
                                    0 => MenuButton::Play,
                                    1 => MenuButton::LevelSelect,
                                    2 => MenuButton::Credits,
                                    3 => MenuButton::Quit,
                                    _ => unreachable!(),
                                },
                            ))
                            .with_children(|p| {
                                p.spawn(TextBundle {
                                    text: Text::from_section(
                                        match i {
                                            0 => "Play",
                                            1 => "Select Level",
                                            2 => "Credits",
                                            3 => "Quit",
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

#[derive(Component)]
struct SpawnedPoints;

fn spawn_title_points(
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    window: Query<&Window>,
    mut commands: Commands,
    mut png: Local<Option<(Handle<Image>, Handle<Image>)>>,
    done: Query<Entity, With<SpawnedPoints>>,
) {
    if png.is_none() {
        *png = Some((
            asset_server.load("title-1.png"),
            asset_server.load("title-2.png"),
        ));
        return;
    }

    if done.get_single().is_ok() {
        return;
    }
    let Some(image_1) = images.get(&png.as_ref().unwrap().0) else {
        return;
    };
    let Some(image_2) = images.get(&png.as_ref().unwrap().1) else {
        return;
    };

    let resolution: u32 = 8;
    let mut to_spawn = Vec::with_capacity(561);
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
            color: title_color.with_alpha(0.0),
        }
        .ease_to(
            ImageColor { color: title_color },
            EaseFunction::QuadraticInOut,
            EasingType::Once {
                duration: point_to_image_duration,
            },
        )
        .delay(point_placement_duration)
        .with_original_value(),
        StateScoped(CURRENT_STATE),
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
            to_spawn.push((
                NodeBundle {
                    z_index: ZIndex::Global(0),
                    border_radius: BorderRadius::MAX,
                    background_color: BackgroundColor(title_color.with_alpha(0.0).into()),
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
                BackgroundColor(title_color.with_alpha(0.0).into())
                    .ease_to(
                        BackgroundColor(title_color.into()),
                        EaseFunction::QuadraticInOut,
                        EasingType::Once {
                            duration: Duration::from_secs_f32(0.5),
                        },
                    )
                    .ease_to(
                        BackgroundColor(title_color.with_alpha(0.0).into()),
                        EaseFunction::QuadraticInOut,
                        EasingType::Once {
                            duration: point_to_image_duration,
                        },
                    )
                    .delay(point_placement_duration - Duration::from_secs_f32(0.5)),
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
            color: title_color.with_alpha(0.0),
        }
        .ease_to(
            ImageColor { color: title_color },
            EaseFunction::QuadraticInOut,
            EasingType::Once {
                duration: point_to_image_duration,
            },
        )
        .delay(point_placement_duration + second_image_delay)
        .with_original_value(),
        StateScoped(CURRENT_STATE),
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
            to_spawn.push((
                NodeBundle {
                    z_index: ZIndex::Global(0),
                    border_radius: BorderRadius::MAX,
                    background_color: BackgroundColor(title_color.with_alpha(0.0).into()),
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
                BackgroundColor(title_color.with_alpha(0.0).into())
                    .ease_to(
                        BackgroundColor(title_color.into()),
                        EaseFunction::QuadraticInOut,
                        EasingType::Once {
                            duration: Duration::from_secs_f32(0.5),
                        },
                    )
                    .ease_to(
                        BackgroundColor(title_color.with_alpha(0.0).into()),
                        EaseFunction::QuadraticInOut,
                        EasingType::Once {
                            duration: point_to_image_duration,
                        },
                    )
                    .delay(
                        point_placement_duration + second_image_delay
                            - Duration::from_secs_f32(0.5),
                    ),
                Dot,
            ));
        }
    }

    commands.spawn_batch(to_spawn);

    commands.spawn((SpawnedPoints, StateScoped(CURRENT_STATE)));
}

fn spawn_reverse_title_points(
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    window: Query<&Window>,
    mut commands: Commands,
    image_query: Query<Entity, With<ImageColor>>,
    mut event_reader: EventReader<SwitchState>,
) {
    if event_reader.read().last().is_none() {
        return;
    }
    let png = (
        asset_server.load("title-1.png"),
        asset_server.load("title-2.png"),
    );

    let Some(image_1) = images.get(&png.0) else {
        return;
    };
    let Some(image_2) = images.get(&png.1) else {
        return;
    };

    let resolution: u32 = 8;
    let mut to_spawn = Vec::with_capacity(561);
    let window_size = window.single().size();

    let point_to_image_duration = Duration::from_secs_f32(0.2);
    let point_placement_duration = Duration::from_secs_f32(0.75);

    let title_color = palettes::tailwind::GREEN_400;

    for entity in &image_query {
        commands.entity(entity).insert(
            ImageColor { color: title_color }
                .ease_to(
                    ImageColor {
                        color: title_color.with_alpha(0.0),
                    },
                    EaseFunction::QuadraticInOut,
                    EasingType::Once {
                        duration: point_to_image_duration,
                    },
                )
                .with_original_value(),
        );
    }

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
            to_spawn.push((
                NodeBundle {
                    z_index: ZIndex::Global(0),
                    border_radius: BorderRadius::MAX,
                    background_color: BackgroundColor(title_color.into()),
                    ..Default::default()
                },
                Style {
                    width: Val::Px(resolution as f32 / 2.0),
                    height: Val::Px(resolution as f32 / 2.0),
                    left: image_1_position(i, j).0,
                    top: image_1_position(i, j).1,
                    position_type: PositionType::Absolute,
                    ..Default::default()
                }
                .ease_to(
                    Style {
                        width: Val::Px(resolution as f32 / 2.0),
                        height: Val::Px(resolution as f32 / 2.0),
                        left: Val::Px(rand::thread_rng().gen_range(0.0..window_size.x)),
                        top: Val::Px(rand::thread_rng().gen_range(0.0..window_size.y)),
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
                StateScoped(CURRENT_STATE),
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
                left: image_2_position(i, j).0,
                top: image_2_position(i, j).1,
                position_type: PositionType::Absolute,
                ..Default::default()
            };
            to_spawn.push((
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
                        left: Val::Px(rand::thread_rng().gen_range(0.0..window_size.x)),
                        top: Val::Px(rand::thread_rng().gen_range(0.0..window_size.y)),
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
                StateScoped(CURRENT_STATE),
            ));
        }
    }

    commands.spawn_batch(to_spawn);
}

#[derive(Component, PartialEq, Eq)]
enum MenuItem {
    Root,
    Panel,
    Button,
}

#[derive(Component)]
struct Dot;

#[derive(Component)]
enum MenuButton {
    Play,
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
    dots: Query<&BackgroundColor, With<Dot>>,
) {
    // Update color of images
    for (color, mut image) in query.iter_mut() {
        if color.is_changed() {
            image.color = color.color.into();
        }
    }
    // Despawn dots once they are finished
    for entity in removed_transitions.read() {
        if dots
            .get(entity)
            .map(|bc| bc.0.alpha() == 0.0)
            .unwrap_or(false)
        {
            commands.entity(entity).despawn();
        }
    }
}

fn button_system(
    mut commands: Commands,
    interaction_query: Query<
        (Ref<Interaction>, &BackgroundColor, &MenuButton, Entity),
        Changed<Interaction>,
    >,
    mut exit: EventWriter<AppExit>,
    mut next_state: EventWriter<SwitchState>,
    ui_items: Query<(Entity, &MenuItem)>,
    camera_position: Query<(Entity, &Transform), With<Camera>>,
    progress: Res<GameProgress>,
) {
    for (interaction, color, button, entity) in &interaction_query {
        if interaction.is_added() {
            continue;
        }
        match *interaction {
            Interaction::Pressed => {
                match button {
                    MenuButton::Play => {
                        next_state.send(SwitchState(GameState::Play(progress.current_level)));

                        let (entity, transform) = camera_position.single();
                        commands.entity(entity).insert(transform.ease_to(
                            Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
                            EaseFunction::QuadraticInOut,
                            EasingType::Once {
                                duration: Duration::from_secs_f32(1.0),
                            },
                        ));

                        for (entity, kind) in &ui_items {
                            if *kind == MenuItem::Root {
                                commands.entity(entity).insert(
                                    Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        left: Val::Percent(0.0),
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
                    }
                    MenuButton::LevelSelect => {
                        next_state.send(SwitchState(GameState::LevelSelect));

                        let (entity, transform) = camera_position.single();
                        commands.entity(entity).insert(transform.ease_to(
                            Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
                            EaseFunction::QuadraticInOut,
                            EasingType::Once {
                                duration: Duration::from_secs_f32(1.0),
                            },
                        ));

                        for (entity, kind) in &ui_items {
                            if *kind == MenuItem::Root {
                                commands.entity(entity).insert(
                                    Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        left: Val::Percent(0.0),
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
                    }
                    MenuButton::Credits => {
                        next_state.send(SwitchState(GameState::Credits));

                        let (entity, transform) = camera_position.single();
                        commands.entity(entity).insert(transform.ease_to(
                            Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
                            EaseFunction::QuadraticInOut,
                            EasingType::Once {
                                duration: Duration::from_secs_f32(1.0),
                            },
                        ));

                        for (entity, kind) in &ui_items {
                            if *kind == MenuItem::Root {
                                commands.entity(entity).insert(
                                    Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        left: Val::Percent(0.0),
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
                    }
                    MenuButton::Quit => {
                        exit.send_default();
                    }
                };
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

#[derive(Event)]
pub struct SwitchState(pub GameState);

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

#[cfg(feature = "debug")]
fn display_navmesh(navmesh: Res<NavMesh>, mut gizmos: Gizmos) {
    use bevy::math::vec3;
    let mesh = navmesh.0.get();
    for polygon in &mesh.polygons {
        let mut v = polygon
            .vertices
            .iter()
            .map(|i| &mesh.vertices[*i as usize].coords)
            .map(|v| vec3(v.x, 0.3, v.y))
            .collect::<Vec<_>>();
        if !v.is_empty() {
            let first = polygon.vertices[0];
            let first = &mesh.vertices[first as usize];
            v.push(vec3(first.coords.x, 0.3, first.coords.y));
            gizmos.linestrip(v, palettes::tailwind::RED_800);
        }
    }
}
