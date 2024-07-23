use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, FRAC_PI_8, PI},
    time::Duration,
};

use avian3d::{collision::Collider, prelude::RigidBody};
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    color::palettes,
    math::{vec2, vec3, CompassQuadrant},
    prelude::*,
    reflect::TypePath,
    scene::{SceneInstance, SceneInstanceReady},
};
use bevy_easings::{CustomComponentEase, EaseFunction, EasingType};
use bevy_firework::{
    bevy_utilitarian::{
        prelude::{Gradient, ParamCurve},
        randomized_values::{RandF32, RandValue, RandVec3},
    },
    core::{BlendMode, ParticleSpawnerBundle, ParticleSpawnerSettings},
    emission_shape::EmissionShape,
};
use bitflags::bitflags;
use polyanya::Polygon;
use thiserror::Error;

use crate::assets::GameAssets;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Tile {
    Start,
    Floor,
    Chest(CompassQuadrant),
    Empty,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct Level {
    pub floors: Vec<Vec<Vec<Tile>>>,
    pub neighbours: Vec<Vec<Vec<Flags>>>,
    pub start: (usize, usize, usize),
    pub end: (usize, usize, usize),
    pub nb_hobbits: u32,
    pub spawn_delay: f32,
}

#[derive(Default)]
struct LevelAssetLoader;

/// Possible errors that can be produced by [`BlobAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
enum LevelAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for LevelAssetLoader {
    type Asset = Level;
    type Settings = ();
    type Error = LevelAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut content = String::new();
        reader.read_to_string(&mut content).await?;
        let mut floor = Vec::new();
        let mut start = (0, 0, 0);
        let mut end = (0, 0, 0);

        let mut lines = content.lines();
        let line = lines.next().unwrap();
        let nb_hobbits = line.split(":").last().unwrap().parse().unwrap();
        let line = lines.next().unwrap();
        let spawn_delay = line.split(":").last().unwrap().parse().unwrap();

        for (j, line) in lines.enumerate() {
            let mut row = Vec::new();
            for (i, char) in line.chars().enumerate() {
                row.push(match char {
                    'X' => {
                        start.1 = i;
                        start.2 = j;
                        Tile::Start
                    }
                    '#' => Tile::Floor,

                    '<' => {
                        end.1 = i;
                        end.2 = j;
                        Tile::Chest(CompassQuadrant::West)
                    }
                    '^' => {
                        end.1 = i;
                        end.2 = j;
                        Tile::Chest(CompassQuadrant::North)
                    }
                    '>' => {
                        end.1 = i;
                        end.2 = j;
                        Tile::Chest(CompassQuadrant::East)
                    }
                    'v' => {
                        end.1 = i;
                        end.2 = j;
                        Tile::Chest(CompassQuadrant::South)
                    }
                    ' ' => Tile::Empty,
                    _ => unimplemented!(),
                });
            }
            floor.push(row);
        }

        let mut neighbours = Vec::new();
        for j in 0..floor.len() {
            let mut row = Vec::new();
            for i in 0..floor[0].len() {
                row.push(get_neighbours(0, i, j, &vec![&floor]));
            }
            neighbours.push(row);
        }

        Ok(Level {
            floors: vec![floor],
            neighbours: vec![neighbours],
            start,
            end,
            nb_hobbits,
            spawn_delay,
        })
    }
}

fn get_neighbours(floor: usize, i: usize, j: usize, data: &Vec<&Vec<Vec<Tile>>>) -> Flags {
    let floor = &data[floor];
    let mut flags = Flags::empty();
    if floor
        .get(j - 1)
        .and_then(|row| row.get(i - 1))
        .map(|t| t != &Tile::Empty)
        .unwrap_or(false)
    {
        flags |= Flags::TOPLEFT;
    }
    if floor
        .get(j)
        .and_then(|row| row.get(i - 1))
        .map(|t| t != &Tile::Empty)
        .unwrap_or(false)
    {
        flags |= Flags::LEFT;
    }
    if floor
        .get(j + 1)
        .and_then(|row| row.get(i - 1))
        .map(|t| t != &Tile::Empty)
        .unwrap_or(false)
    {
        flags |= Flags::BOTTOMLEFT;
    }
    if floor
        .get(j - 1)
        .and_then(|row| row.get(i))
        .map(|t| t != &Tile::Empty)
        .unwrap_or(false)
    {
        flags |= Flags::TOP;
    }
    if floor
        .get(j)
        .and_then(|row| row.get(i))
        .map(|t| t != &Tile::Empty)
        .unwrap_or(false)
    {
        flags |= Flags::CENTER;
    }
    if floor
        .get(j + 1)
        .and_then(|row| row.get(i))
        .map(|t| t != &Tile::Empty)
        .unwrap_or(false)
    {
        flags |= Flags::BOTTOM;
    }
    if floor
        .get(j - 1)
        .and_then(|row| row.get(i + 1))
        .map(|t| t != &Tile::Empty)
        .unwrap_or(false)
    {
        flags |= Flags::TOPRIGHT;
    }
    if floor
        .get(j)
        .and_then(|row| row.get(i + 1))
        .map(|t| t != &Tile::Empty)
        .unwrap_or(false)
    {
        flags |= Flags::RIGHT;
    }
    if floor
        .get(j + 1)
        .and_then(|row| row.get(i + 1))
        .map(|t| t != &Tile::Empty)
        .unwrap_or(false)
    {
        flags |= Flags::BOTTOMRIGHT;
    }
    flags
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Level>()
            .init_asset_loader::<LevelAssetLoader>()
            .add_systems(
                Update,
                (
                    open_lid,
                    handle_light,
                    bevy_easings::custom_ease_system::<PointLightIntensity>,
                    bevy_easings::custom_ease_system::<DirectionalLightIlluminance>,
                ),
            );
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Flags: u32 {
        const CENTER = 0b000000001;
        const TOP = 0b000000010;
        const BOTTOM = 0b000000100;
        const LEFT = 0b000001000;
        const RIGHT = 0b000010000;
        const TOPLEFT = 0b000100000;
        const TOPRIGHT = 0b001000000;
        const BOTTOMLEFT = 0b010000000;
        const BOTTOMRIGHT = 0b100000000;
    }
}

pub fn spawn_level(
    commands: &mut Commands,
    level: &Level,
    assets: &GameAssets,
    tag: impl Component,
    directional_light: (Entity, &mut DirectionalLight),
) -> ((usize, usize), polyanya::Mesh) {
    directional_light.1.illuminance = 0.0;
    commands
        .entity(directional_light.0)
        .insert(DirectionalLightIlluminance(0.0).ease_to(
            DirectionalLightIlluminance(light_consts::lux::OFFICE),
            EaseFunction::QuadraticOut,
            EasingType::Once {
                duration: Duration::from_secs_f32(4.0),
            },
        ));

    let floor = &level.floors[0];
    let mut vertices = Vec::with_capacity((floor.len() + 1) * (floor[0].len() + 1));
    let mut polygons = Vec::with_capacity((floor.len() + 1) * (floor[0].len() + 1) / 2);

    let wall_scale = vec3(1.0, 0.5, 0.25);
    let corner_scale = vec3(0.25, 0.5, 0.25);

    let mut polygon_holes = vec![];

    commands
        .spawn((SpatialBundle::default(), tag))
        .with_children(|parent| {
            let floor = &level.floors[0];
            for (yi, row) in floor.iter().enumerate() {
                for (xi, tile) in row.iter().enumerate() {
                    let flag = level.neighbours[0][yi][xi].clone();
                    let x = xi as f32 * 4.0;
                    let y = yi as f32 * 4.0;

                    if flag.contains(Flags::CENTER) {
                        if !flag.contains(Flags::TOP) {
                            parent.spawn(SceneBundle {
                                scene: assets.wall.clone(),
                                transform: Transform::from_translation(Vec3::new(x, 0.0, y - 2.0))
                                    .with_scale(wall_scale),
                                ..default()
                            });
                        }
                        if !flag.contains(Flags::BOTTOM) {
                            parent.spawn(SceneBundle {
                                scene: assets.wall.clone(),
                                transform: Transform::from_translation(Vec3::new(x, 0.0, y + 2.0))
                                    .with_scale(wall_scale),
                                ..default()
                            });
                        }
                        if !flag.contains(Flags::LEFT) {
                            parent.spawn(SceneBundle {
                                scene: assets.wall.clone(),
                                transform: Transform::from_translation(Vec3::new(x - 2.0, 0.0, y))
                                    .with_rotation(Quat::from_rotation_y(FRAC_PI_2))
                                    .with_scale(wall_scale),
                                ..default()
                            });
                        }
                        if !flag.contains(Flags::RIGHT) {
                            parent.spawn(SceneBundle {
                                scene: assets.wall.clone(),
                                transform: Transform::from_translation(Vec3::new(x + 2.0, 0.0, y))
                                    .with_rotation(Quat::from_rotation_y(FRAC_PI_2))
                                    .with_scale(wall_scale),
                                ..default()
                            });
                        }
                        if !flag.contains(Flags::TOP) && !flag.contains(Flags::LEFT) {
                            parent.spawn(SceneBundle {
                                scene: assets.wall_corner.clone(),
                                transform: Transform::from_translation(Vec3::new(
                                    x - 2.0,
                                    0.0,
                                    y - 2.0,
                                ))
                                .with_rotation(Quat::from_rotation_y(FRAC_PI_2))
                                .with_scale(corner_scale),
                                ..default()
                            });
                        }
                        if !flag.contains(Flags::TOP) && !flag.contains(Flags::RIGHT) {
                            parent.spawn(SceneBundle {
                                scene: assets.wall_corner.clone(),
                                transform: Transform::from_translation(Vec3::new(
                                    x + 2.0,
                                    0.0,
                                    y - 2.0,
                                ))
                                .with_scale(corner_scale),
                                ..default()
                            });
                        }
                        if !flag.contains(Flags::BOTTOM) && !flag.contains(Flags::LEFT) {
                            parent.spawn(SceneBundle {
                                scene: assets.wall_corner.clone(),
                                transform: Transform::from_translation(Vec3::new(
                                    x - 2.0,
                                    0.0,
                                    y + 2.0,
                                ))
                                .with_rotation(Quat::from_rotation_y(PI))
                                .with_scale(corner_scale),
                                ..default()
                            });
                        }
                        if !flag.contains(Flags::BOTTOM) && !flag.contains(Flags::RIGHT) {
                            parent.spawn(SceneBundle {
                                scene: assets.wall_corner.clone(),
                                transform: Transform::from_translation(Vec3::new(
                                    x + 2.0,
                                    0.0,
                                    y + 2.0,
                                ))
                                .with_rotation(Quat::from_rotation_y(-FRAC_PI_2))
                                .with_scale(corner_scale),
                                ..default()
                            });
                        }
                    }

                    match tile {
                        Tile::Start => {
                            parent.spawn((
                                PointLightBundle {
                                    transform: Transform::from_translation(Vec3::new(x, 5.0, y)),
                                    point_light: PointLight {
                                        intensity: 0.0,
                                        shadows_enabled: true,
                                        range: 20.0,
                                        ..default()
                                    },

                                    ..default()
                                },
                                PointLightIntensity(0.0),
                                PointLightIntensity(0.0).ease_to(
                                    PointLightIntensity(1_500_000.0),
                                    EaseFunction::QuadraticOut,
                                    EasingType::Once {
                                        duration: Duration::from_secs_f32(3.0),
                                    },
                                ),
                            ));
                            parent.spawn((
                                SceneBundle {
                                    scene: assets.floor.clone(),
                                    transform: Transform::from_translation(Vec3::new(x, 0.0, y)),
                                    ..default()
                                },
                                RigidBody::Static,
                                Collider::cuboid(4.0, 0.2, 4.0),
                            ));
                            parent
                                .spawn(ParticleSpawnerBundle::from_settings(
                                    ParticleSpawnerSettings {
                                        one_shot: false,
                                        rate: 5000.0,
                                        emission_shape: EmissionShape::Circle {
                                            normal: Vec3::Y,
                                            radius: 1.5,
                                        },
                                        lifetime: RandF32::constant(0.25),
                                        inherit_parent_velocity: true,
                                        initial_velocity: RandVec3 {
                                            magnitude: RandF32 { min: 0., max: 10. },
                                            direction: Vec3::Y,
                                            spread: FRAC_PI_8,
                                        },
                                        initial_scale: RandF32 {
                                            min: 0.02,
                                            max: 0.08,
                                        },
                                        scale_curve: ParamCurve::constant(1.),
                                        color: Gradient::linear(vec![
                                            (0., LinearRgba::new(150., 100., 15., 1.)),
                                            (0.7, LinearRgba::new(3., 1., 1., 1.)),
                                            (0.8, LinearRgba::new(1., 0.3, 0.3, 1.)),
                                            (0.9, LinearRgba::new(0.3, 0.3, 0.3, 1.)),
                                            (1., LinearRgba::new(0.1, 0.1, 0.1, 0.)),
                                        ]),
                                        blend_mode: BlendMode::Blend,
                                        linear_drag: 0.1,
                                        pbr: false,
                                        ..default()
                                    },
                                ))
                                .insert(Transform::from_translation(Vec3::new(x, 0.0, y)));
                        }
                        Tile::Floor => {
                            parent.spawn((
                                SceneBundle {
                                    scene: assets.floor.clone(),
                                    transform: Transform::from_translation(Vec3::new(x, 0.0, y)),
                                    ..default()
                                },
                                RigidBody::Static,
                                Collider::cuboid(4.0, 0.2, 4.0),
                            ));
                        }
                        Tile::Chest(direction) => {
                            parent.spawn((
                                PointLightBundle {
                                    transform: Transform::from_translation(Vec3::new(x, 5.0, y)),
                                    point_light: PointLight {
                                        intensity: 0.0,
                                        color: palettes::tailwind::YELLOW_800.into(),
                                        shadows_enabled: true,
                                        ..default()
                                    },
                                    ..default()
                                },
                                PointLightIntensity(0.0),
                                PointLightIntensity(0.0)
                                    .ease_to(
                                        PointLightIntensity(1_000_000.0),
                                        EaseFunction::QuadraticOut,
                                        EasingType::Once {
                                            duration: Duration::from_secs_f32(3.0),
                                        },
                                    )
                                    .ease_to(
                                        PointLightIntensity(1_250_000.0),
                                        EaseFunction::QuadraticInOut,
                                        EasingType::PingPong {
                                            duration: Duration::from_secs_f32(1.0),
                                            pause: None,
                                        },
                                    ),
                            ));
                            parent.spawn((
                                SceneBundle {
                                    scene: assets.floor.clone(),
                                    transform: Transform::from_translation(Vec3::new(x, 0.0, y)),
                                    ..default()
                                },
                                RigidBody::Static,
                                Collider::cuboid(4.0, 0.2, 4.0),
                            ));
                            parent
                                .spawn(SpatialBundle {
                                    transform: Transform::from_translation(Vec3::new(x, 0.0, y))
                                        .with_rotation(match direction {
                                            CompassQuadrant::North => Quat::from_rotation_y(PI),
                                            CompassQuadrant::East => {
                                                Quat::from_rotation_y(FRAC_PI_2)
                                            }
                                            CompassQuadrant::South => Quat::IDENTITY,
                                            CompassQuadrant::West => {
                                                Quat::from_rotation_y(-FRAC_PI_2)
                                            }
                                        }),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(SceneBundle {
                                        scene: assets.chest.clone(),
                                        ..default()
                                    });
                                    parent.spawn(SceneBundle {
                                        scene: assets.coin_stack.clone(),
                                        transform: Transform::from_translation(Vec3::new(
                                            1.0, 0.0, 0.0,
                                        )),
                                        ..default()
                                    });
                                    parent.spawn(SceneBundle {
                                        scene: assets.coin_stack.clone(),
                                        transform: Transform::from_translation(Vec3::new(
                                            -1.5, 0.0, 0.0,
                                        )),
                                        ..default()
                                    });
                                    parent.spawn(ParticleSpawnerBundle::from_settings(
                                        ParticleSpawnerSettings {
                                            one_shot: false,
                                            rate: 10.0,
                                            emission_shape: EmissionShape::Circle {
                                                normal: Vec3::Y,
                                                radius: 0.5,
                                            },
                                            lifetime: RandF32::constant(0.25),
                                            inherit_parent_velocity: true,
                                            initial_velocity: RandVec3 {
                                                magnitude: RandF32 { min: 0., max: 10. },
                                                direction: Vec3::Y,
                                                spread: FRAC_PI_4,
                                            },
                                            initial_scale: RandF32 {
                                                min: 0.05,
                                                max: 0.1,
                                            },
                                            scale_curve: ParamCurve::constant(1.),
                                            color: Gradient::constant(
                                                (palettes::tailwind::YELLOW_800 * 10.0).into(),
                                            ),
                                            blend_mode: BlendMode::Blend,
                                            linear_drag: 0.1,
                                            pbr: true,
                                            ..default()
                                        },
                                    ));
                                });
                        }
                        Tile::Empty => {}
                    }
                    let mut delta_x = 0.0;
                    let mut delta_y = 0.0;
                    if !flag.contains(Flags::TOP) {
                        delta_y += 1.0;
                    }
                    if flag.contains(Flags::CENTER) && !flag.contains(Flags::LEFT) {
                        delta_x += 1.0;
                    }
                    if flag.contains(Flags::CENTER)
                        && !flag.contains(Flags::LEFT)
                        && flag.contains(Flags::TOPLEFT)
                    {
                        delta_y -= 1.0;
                    }
                    if !flag.contains(Flags::CENTER)
                        && flag.contains(Flags::LEFT)
                        && !flag.contains(Flags::TOPLEFT)
                    {
                        delta_y += 1.0;
                    }
                    if !flag.contains(Flags::CENTER) && flag.contains(Flags::LEFT) {
                        delta_x -= 1.0;
                    }
                    if !flag.contains(Flags::TOP)
                        && flag.contains(Flags::LEFT)
                        && flag.contains(Flags::TOPLEFT)
                    {
                        delta_x -= 1.0;
                    }
                    if flag.contains(Flags::CENTER)
                        && flag.contains(Flags::TOP)
                        && flag.contains(Flags::LEFT)
                        && !flag.contains(Flags::TOPLEFT)
                    {
                        delta_y += 1.0;
                        delta_x += 1.0;
                    }
                    if !flag.contains(Flags::CENTER) {
                        delta_y -= 1.0;
                    }

                    if tile == &Tile::Empty {
                        polygon_holes.push((xi + row.len() * yi) as isize);
                    }

                    let spatial_to_index = |index: isize| {
                        let holes_before = polygon_holes.iter().filter(|i| i < &&index).count();
                        index - holes_before as isize
                    };

                    let mut neighbours = vec![];
                    if flag.contains(Flags::CENTER) {
                        neighbours.push(spatial_to_index((xi + row.len() * yi) as isize));
                    } else {
                        neighbours.push(-1);
                    }
                    if flag.contains(Flags::LEFT) {
                        neighbours.push(spatial_to_index(((xi - 1) + row.len() * yi) as isize));
                    } else if !neighbours.contains(&-1) {
                        neighbours.push(-1);
                    }
                    if flag.contains(Flags::TOP) {
                        neighbours.push(spatial_to_index((xi + row.len() * (yi - 1)) as isize));
                    } else if !neighbours.contains(&-1) {
                        neighbours.push(-1);
                    }
                    if flag.contains(Flags::TOPLEFT) {
                        neighbours
                            .push(spatial_to_index(((xi - 1) + row.len() * (yi - 1)) as isize));
                    } else if !neighbours.contains(&-1) {
                        neighbours.push(-1);
                    }

                    vertices.push(polyanya::Vertex::new(
                        vec2(
                            xi as f32 * 4.0 - 2.0 + delta_x,
                            yi as f32 * 4.0 - 2.0 + delta_y,
                        ),
                        neighbours,
                    ));
                    if tile != &Tile::Empty {
                        polygons.push(Polygon::new(
                            vec![
                                (xi + (row.len() + 1) * yi) as u32,
                                (xi + (row.len() + 1) * (yi + 1)) as u32,
                                (xi + 1 + (row.len() + 1) * (yi + 1)) as u32,
                                (xi + 1 + (row.len() + 1) * yi) as u32,
                            ],
                            false,
                        ));
                    }
                }
                let mut delta_y = 0.0;

                if yi == 0 {
                    delta_y += 1.0;
                }
                vertices.push(polyanya::Vertex::new(
                    vec2(
                        row.len() as f32 * 4.0 - 2.0 - 1.0,
                        yi as f32 * 4.0 - 2.0 + delta_y,
                    ),
                    vec![-1],
                ));
            }
            for xi in 0..floor[0].len() {
                let flag = level.neighbours[0][floor.len() - 1][xi].clone();
                let mut delta_x = 0.0;
                if !flag.contains(Flags::CENTER) {
                    delta_x -= 1.0;
                }
                if flag.contains(Flags::CENTER) && !flag.contains(Flags::LEFT) {
                    delta_x += 1.0;
                }
                vertices.push(polyanya::Vertex::new(
                    vec2(
                        xi as f32 * 4.0 - 2.0 + delta_x,
                        floor.len() as f32 * 4.0 - 2.0 - 1.0,
                    ),
                    vec![-1],
                ));
            }
            vertices.push(polyanya::Vertex::new(
                vec2(
                    floor[0].len() as f32 * 4.0 - 2.0 - 1.0,
                    floor.len() as f32 * 4.0 - 2.0 - 1.0,
                ),
                vec![-1],
            ));
        });

    let mesh = polyanya::Mesh::new(vertices, polygons).unwrap();
    (
        (level.floors[0].len() * 4, level.floors[0][0].len() * 4),
        mesh,
    )
}

fn open_lid(
    mut scenes_loaded: EventReader<SceneInstanceReady>,
    scene_instances: Query<&SceneInstance>,
    mut transforms: Query<(&Name, &mut Transform)>,
    scene_spawner: Res<SceneSpawner>,
    has_material: Query<&Handle<StandardMaterial>>,
) {
    let lid_name = Name::new("chest_gold_lid");
    for scene in scenes_loaded.read() {
        let scene_instance = scene_instances.get(scene.parent).unwrap();
        scene_spawner
            .iter_instance_entities(**scene_instance)
            .for_each(|e| {
                if let Ok((name, mut transform)) = transforms.get_mut(e) {
                    if *name == lid_name && has_material.get(e).is_err() {
                        transform.rotation = Quat::from_rotation_x(-FRAC_PI_3 * 2.0);
                    }
                }
            });
    }
}

#[derive(Component, Default, Clone, Copy)]
struct PointLightIntensity(f32);
impl bevy_easings::Lerp for PointLightIntensity {
    type Scalar = f32;

    fn lerp(&self, other: &Self, scalar: &Self::Scalar) -> Self {
        PointLightIntensity(self.0.lerp(other.0, *scalar))
    }
}
#[derive(Component, Default, Clone, Copy)]
pub struct DirectionalLightIlluminance(pub f32);
impl bevy_easings::Lerp for DirectionalLightIlluminance {
    type Scalar = f32;

    fn lerp(&self, other: &Self, scalar: &Self::Scalar) -> Self {
        DirectionalLightIlluminance(self.0.lerp(other.0, *scalar))
    }
}

fn handle_light(
    mut point_lights: Query<(Ref<PointLightIntensity>, &mut PointLight)>,
    mut directional_lights: Query<(Ref<DirectionalLightIlluminance>, &mut DirectionalLight)>,
) {
    for (intensity, mut light) in point_lights.iter_mut() {
        if intensity.is_changed() {
            light.intensity = intensity.0;
        }
    }
    for (illuminance, mut light) in directional_lights.iter_mut() {
        if illuminance.is_changed() {
            light.illuminance = illuminance.0;
        }
    }
}
