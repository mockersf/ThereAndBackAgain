//! Implements loader for a custom asset type.

use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_3, PI},
    time::Duration,
};

use avian3d::{collision::Collider, prelude::RigidBody};
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    color::palettes,
    math::CompassQuadrant,
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
use thiserror::Error;

use crate::assets::GameAssets;

#[derive(Debug)]
pub enum Tile {
    Start,
    Floor,
    Chest(CompassQuadrant),
    Empty,
}

#[derive(Asset, TypePath, Debug)]
pub struct Level {
    pub floors: Vec<Vec<Vec<Tile>>>,
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
        let mut level = Vec::new();
        for line in content.lines() {
            let mut row = Vec::new();
            for char in line.chars() {
                row.push(match char {
                    'X' => Tile::Start,
                    '#' => Tile::Floor,
                    '<' => Tile::Chest(CompassQuadrant::West),
                    '^' => Tile::Chest(CompassQuadrant::North),
                    '>' => Tile::Chest(CompassQuadrant::East),
                    'v' => Tile::Chest(CompassQuadrant::South),
                    ' ' => Tile::Empty,
                    _ => unimplemented!(),
                });
            }
            level.push(row);
        }

        Ok(Level {
            floors: vec![level],
        })
    }
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

pub fn spawn_level(
    commands: &mut Commands,
    level: &Level,
    assets: &GameAssets,
    tag: impl Component,
    directional_light: (Entity, &mut DirectionalLight),
) -> (usize, usize) {
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
    commands
        .spawn((SpatialBundle::default(), tag))
        .with_children(|parent| {
            for (y, row) in level.floors[0].iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    let x = x as f32 * 4.0;
                    let y = y as f32 * 4.0;

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
                                            spread: 30. / 180. * PI,
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
                                });
                        }
                        Tile::Empty => {}
                    }
                }
            }
        });
    (level.floors[0].len() * 4, level.floors[0][0].len() * 4)
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
                    if *name == lid_name && !has_material.get(e).is_ok() {
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
