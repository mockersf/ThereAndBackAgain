use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, FRAC_PI_8, PI};

use avian3d::{collision::Collider, prelude::RigidBody};
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    color::palettes,
    math::{uvec2, vec2, vec3, CompassQuadrant},
    prelude::*,
    reflect::TypePath,
    scene::{SceneInstance, SceneInstanceReady},
};
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
    In,
    Out,
    Empty,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Bonus {
    Obstacle,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct Level {
    pub floors: Vec<Vec<Vec<Tile>>>,
    pub neighbours: Vec<Vec<Vec<Flags>>>,
    pub start: (usize, usize, usize),
    pub end: (usize, usize, usize),
    pub nb_hobbits: u32,
    pub spawn_delay: f32,
    pub message: Option<String>,
    pub goal: Option<String>,
    pub treasures: u32,
    pub losts: Option<u32>,
    pub bonus: Vec<Bonus>,
    pub file: String,
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
        let nb_hobbits = line.split(':').last().unwrap().parse().unwrap();
        let line = lines.next().unwrap();
        let spawn_delay = line.split(':').last().unwrap().parse().unwrap();
        let line = lines.next().unwrap();
        let message = line
            .split(':')
            .skip(1)
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(":");
        let message = if &message == "none" {
            None
        } else {
            Some(message.replace("\\n", "\n"))
        };

        let line = lines.next().unwrap();
        let goal = match line.split(':').last().unwrap() {
            "none" => None,
            s => Some(s.to_string()),
        };
        let line = lines.next().unwrap();
        let treasures = line.split(':').last().unwrap().parse().unwrap();
        let line = lines.next().unwrap();
        let losts = line.split(':').last().unwrap().parse().ok();
        let line = lines.next().unwrap();
        let bonus = line
            .split(':')
            .last()
            .unwrap()
            .split(',')
            .flat_map(|s| match s {
                "Obstacle" => Some(Bonus::Obstacle),
                "" => None,
                s => {
                    error!("unknown bonus: {}", s);
                    unimplemented!()
                }
            })
            .collect::<Vec<_>>();

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
                    'I' => Tile::In,
                    'O' => Tile::Out,
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
                row.push(get_neighbours(0, i, j, &[&floor]));
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
            message,
            goal,
            treasures,
            losts,
            bonus,
            file: _load_context.path().to_string_lossy().to_string(),
        })
    }

    fn extensions(&self) -> &[&str] {
        &["level"]
    }
}

fn get_neighbours(floor: usize, i: usize, j: usize, data: &[&Vec<Vec<Tile>>]) -> Flags {
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
            .add_systems(Update, (open_lid,));
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

fn fix_indexes(
    mut polygons: Vec<polyanya::Polygon>,
    mut vertices: Vec<polyanya::Vertex>,
    width: u32,
) -> Option<polyanya::Layer> {
    // find polygon of each vertex
    for (poly_index, polygon) in polygons.iter_mut().enumerate() {
        for vertex_index in &polygon.vertices {
            let vertex = &mut vertices[*vertex_index as usize];
            vertex.polygons.push(poly_index as u32);
        }
    }
    // reorder polygons CCW
    let vertices = vertices
        .iter()
        .enumerate()
        .map(|(vertex_index, vertex)| {
            let polys = vertex
                .polygons
                .iter()
                .map(|poly_index| {
                    (
                        poly_index,
                        &vertices[polygons[*poly_index as usize].vertices[0] as usize].coords,
                    )
                })
                .collect::<Vec<_>>();
            let coords_theoretical =
                uvec2(vertex_index as u32 % width, vertex_index as u32 / width).as_vec2() * 4.0
                    - vec2(2.0, 2.0);
            let ccw_order = [(false, false), (false, true), (true, true), (true, false)];

            let mut polys = ccw_order
                .iter()
                .map(|neighbour| {
                    polys
                        .iter()
                        .find(|(_, coords)| match neighbour {
                            (false, false) => {
                                coords.x > coords_theoretical.x + 1.0
                                    && coords.y < coords_theoretical.y - 1.0
                            }
                            (false, true) => {
                                coords.x > coords_theoretical.x + 1.0
                                    && coords.y >= coords_theoretical.y - 1.0
                            }
                            (true, true) => {
                                coords.x <= coords_theoretical.x + 1.0
                                    && coords.y >= coords_theoretical.y - 1.0
                            }
                            (true, false) => {
                                coords.x <= coords_theoretical.x + 1.0
                                    && coords.y < coords_theoretical.y - 1.0
                            }
                        })
                        .map(|(poly_index, _)| **poly_index)
                        .unwrap_or(u32::MAX)
                })
                .collect::<Vec<_>>();
            polys.dedup();
            polys.rotate_left(1);
            polys.dedup();
            polyanya::Vertex::new(vertex.coords, polys)
        })
        .collect();

    polyanya::Layer::new(vertices, polygons)
        .map(|mut layer| {
            layer.remove_useless_vertices();
            layer
        })
        .ok()
}

impl Level {
    pub fn as_navmesh(&self, removed_cells: Vec<(usize, usize)>) -> polyanya::Mesh {
        info!("excluding cells from navmesh: {:?}", removed_cells);
        let floor = &self.floors[0];
        let mut vertices = Vec::with_capacity((floor.len() + 1) * (floor[0].len() + 1) * 4);
        let mut polygons = Vec::with_capacity((floor.len() + 1) * (floor[0].len() + 1) / 2);
        let mut vertices_in = Vec::with_capacity(10);
        let mut polygons_in = Vec::with_capacity(2);
        let mut vertices_out = Vec::with_capacity(10);
        let mut polygons_out = Vec::with_capacity(2);

        let floor = &self.floors[0];
        for (yi, row) in floor.iter().enumerate() {
            for (xi, tile) in row.iter().enumerate() {
                let flag = self.neighbours[0][yi][xi];

                let (delta_x, delta_y) = match (
                    flag.contains(Flags::TOPLEFT),
                    flag.contains(Flags::TOP),
                    flag.contains(Flags::LEFT),
                    flag.contains(Flags::CENTER),
                ) {
                    (true, true, true, true) => (0.0, 0.0),
                    (true, true, true, false) => (-1.0, -1.0),
                    (true, true, false, true) => (1.0, -1.0),
                    (true, true, false, false) => (0.0, -1.0),
                    (true, false, true, true) => (-1.0, 1.0),
                    (true, false, true, false) => (-1.0, 0.0),
                    (true, false, false, true) => {
                        // would need to inject two vertices to differentiate between each side
                        // this would break all the math-based indexing done here
                        unimplemented!("case not handled, design a puzzle without")
                    }
                    (true, false, false, false) => (-1.0, -1.0),
                    (false, true, true, true) => (1.0, 1.0),
                    (false, true, true, false) => {
                        // would need to inject two vertices to differentiate between each side
                        // this would break all the math-based indexing done here
                        unimplemented!("case not handled, design a puzzle without")
                    }
                    (false, true, false, true) => (1.0, 0.0),
                    (false, true, false, false) => (1.0, -1.0),
                    (false, false, true, true) => (0.0, 1.0),
                    (false, false, true, false) => (-1.0, 1.0),
                    (false, false, false, true) => (1.0, 1.0),
                    (false, false, false, false) => (0.0, 0.0),
                };

                vertices.push(polyanya::Vertex::new(
                    vec2(
                        xi as f32 * 4.0 - 2.0 + delta_x,
                        yi as f32 * 4.0 - 2.0 + delta_y,
                    ),
                    vec![],
                ));

                vertices_in.push(polyanya::Vertex::new(
                    vec2(
                        xi as f32 * 4.0 - 2.0 + delta_x,
                        yi as f32 * 4.0 - 2.0 + delta_y,
                    ),
                    vec![],
                ));
                vertices_out.push(polyanya::Vertex::new(
                    vec2(
                        xi as f32 * 4.0 - 2.0 + delta_x,
                        yi as f32 * 4.0 - 2.0 + delta_y,
                    ),
                    vec![],
                ));

                match tile {
                    Tile::In => {
                        polygons_in.push(Polygon::new(
                            vec![
                                (xi + 1 + (row.len() + 1) * yi) as u32,
                                (xi + 1 + (row.len() + 1) * (yi + 1)) as u32,
                                (xi + (row.len() + 1) * (yi + 1)) as u32,
                                (xi + (row.len() + 1) * yi) as u32,
                            ],
                            false,
                        ));
                    }
                    Tile::Out => {
                        polygons_out.push(Polygon::new(
                            vec![
                                (xi + 1 + (row.len() + 1) * yi) as u32,
                                (xi + 1 + (row.len() + 1) * (yi + 1)) as u32,
                                (xi + (row.len() + 1) * (yi + 1)) as u32,
                                (xi + (row.len() + 1) * yi) as u32,
                            ],
                            false,
                        ));
                    }
                    Tile::Empty => (),
                    _ => {
                        if removed_cells.contains(&(xi, yi)) {
                            continue;
                        }
                        polygons.push(Polygon::new(
                            vec![
                                (xi + 1 + (row.len() + 1) * yi) as u32,
                                (xi + 1 + (row.len() + 1) * (yi + 1)) as u32,
                                (xi + (row.len() + 1) * (yi + 1)) as u32,
                                (xi + (row.len() + 1) * yi) as u32,
                            ],
                            false,
                        ));
                    }
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
                vec![],
            ));
            vertices_in.push(polyanya::Vertex::new(
                vec2(
                    row.len() as f32 * 4.0 - 2.0 - 1.0,
                    yi as f32 * 4.0 - 2.0 + delta_y,
                ),
                vec![],
            ));
            vertices_out.push(polyanya::Vertex::new(
                vec2(
                    row.len() as f32 * 4.0 - 2.0 - 1.0,
                    yi as f32 * 4.0 - 2.0 + delta_y,
                ),
                vec![],
            ));
        }
        for xi in 0..floor[0].len() {
            let flag = self.neighbours[0][floor.len() - 1][xi];
            let mut delta_x = 0.0;

            if !flag.contains(Flags::CENTER) {
                delta_x -= 1.0;
            }
            if !flag.contains(Flags::LEFT) && flag.contains(Flags::CENTER) {
                delta_x += 1.0;
            }
            vertices.push(polyanya::Vertex::new(
                vec2(
                    xi as f32 * 4.0 - 2.0 + delta_x,
                    floor.len() as f32 * 4.0 - 2.0 - 1.0,
                ),
                vec![],
            ));

            vertices_in.push(polyanya::Vertex::new(
                vec2(
                    xi as f32 * 4.0 - 2.0 + delta_x,
                    floor.len() as f32 * 4.0 - 2.0 - 1.0,
                ),
                vec![],
            ));
            vertices_out.push(polyanya::Vertex::new(
                vec2(
                    xi as f32 * 4.0 - 2.0 + delta_x,
                    floor.len() as f32 * 4.0 - 2.0 - 1.0,
                ),
                vec![],
            ));
        }
        vertices.push(polyanya::Vertex::new(
            vec2(
                floor[0].len() as f32 * 4.0 - 2.0 - 1.0,
                floor.len() as f32 * 4.0 - 2.0 - 1.0,
            ),
            vec![],
        ));
        vertices_in.push(polyanya::Vertex::new(
            vec2(
                floor[0].len() as f32 * 4.0 - 2.0 - 1.0,
                floor.len() as f32 * 4.0 - 2.0 - 1.0,
            ),
            vec![],
        ));
        vertices_out.push(polyanya::Vertex::new(
            vec2(
                floor[0].len() as f32 * 4.0 - 2.0 - 1.0,
                floor.len() as f32 * 4.0 - 2.0 - 1.0,
            ),
            vec![],
        ));

        let mut layers = vec![];
        let layer = fix_indexes(polygons, vertices, floor[0].len() as u32 + 1).unwrap();
        layers.push(layer);
        if let Some(layer_in) = fix_indexes(polygons_in, vertices_in, floor[0].len() as u32 + 1) {
            layers.push(layer_in);
        } else {
            layers.push(
                polyanya::Layer::new(
                    vec![
                        polyanya::Vertex::new(vec2(-150.0, -150.0), vec![0, u32::MAX]),
                        polyanya::Vertex::new(vec2(-149.99999, -150.0), vec![0, u32::MAX]),
                        polyanya::Vertex::new(vec2(-149.99999, -149.99999), vec![0, u32::MAX]),
                    ],
                    vec![polyanya::Polygon::new(vec![0, 1, 2], false)],
                )
                .unwrap(),
            );
        }
        if let Some(layer_out) = fix_indexes(polygons_out, vertices_out, floor[0].len() as u32 + 1)
        {
            layers.push(layer_out);
        } else {
            layers.push(
                polyanya::Layer::new(
                    vec![
                        polyanya::Vertex::new(vec2(-150.0, -150.0), vec![0, u32::MAX]),
                        polyanya::Vertex::new(vec2(-149.99999, -150.0), vec![0, u32::MAX]),
                        polyanya::Vertex::new(vec2(-149.99999, -149.99999), vec![0, u32::MAX]),
                    ],
                    vec![polyanya::Polygon::new(vec![0, 1, 2], false)],
                )
                .unwrap(),
            );
        }

        let mut mesh = polyanya::Mesh {
            layers,
            ..Default::default()
        };

        if mesh.layers[1].vertices[0].coords.x != -150.0 {
            mesh.restitch_layer_at_points(
                1,
                vec![(
                    (0, 1),
                    mesh.layers[1]
                        .vertices
                        .iter()
                        .map(|v| v.coords)
                        .filter(|coords| {
                            mesh.layers[0].vertices.iter().any(|v| v.coords == *coords)
                        })
                        .collect(),
                )],
            );
        }
        if mesh.layers[2].vertices[0].coords.x != -150.0 {
            mesh.restitch_layer_at_points(
                2,
                vec![(
                    (0, 2),
                    mesh.layers[2]
                        .vertices
                        .iter()
                        .map(|v| v.coords)
                        .filter(|coords| {
                            mesh.layers[0].vertices.iter().any(|v| v.coords == *coords)
                        })
                        .collect(),
                )],
            );
        }

        mesh
    }
}

pub fn spawn_level(
    commands: &mut Commands,
    level: &Level,
    assets: &GameAssets,
    tag: impl Component,
) -> ((usize, usize), polyanya::Mesh) {
    let floor = &level.floors[0];

    let height = if cfg!(feature = "debug") { 0.1 } else { 0.5 };
    let wall_scale = vec3(1.0, height, 0.25);
    let corner_scale = vec3(0.25, height, 0.25);

    commands
        .spawn((SpatialBundle::default(), tag))
        .with_children(|parent| {
            let floor = &floor;
            for (yi, row) in floor.iter().enumerate() {
                for (xi, tile) in row.iter().enumerate() {
                    let flag = level.neighbours[0][yi][xi];
                    let x = xi as f32 * 4.0;
                    let y = yi as f32 * 4.0;

                    if flag.contains(Flags::CENTER) {
                        if !flag.contains(Flags::TOP) {
                            parent.spawn((
                                SceneBundle {
                                    scene: assets.wall.clone(),
                                    transform: Transform::from_translation(Vec3::new(
                                        x,
                                        0.0,
                                        y - 2.0,
                                    ))
                                    .with_scale(wall_scale),
                                    ..default()
                                },
                                RigidBody::Static,
                                Collider::cuboid(4.0, 40.0, 0.2),
                            ));
                        }
                        if !flag.contains(Flags::BOTTOM) {
                            parent.spawn((
                                SceneBundle {
                                    scene: assets.wall.clone(),
                                    transform: Transform::from_translation(Vec3::new(
                                        x,
                                        0.0,
                                        y + 2.0,
                                    ))
                                    .with_scale(wall_scale),
                                    ..default()
                                },
                                RigidBody::Static,
                                Collider::cuboid(4.0, 40.0, 0.2),
                            ));
                        }
                        if !flag.contains(Flags::LEFT) {
                            parent.spawn((
                                SceneBundle {
                                    scene: assets.wall.clone(),
                                    transform: Transform::from_translation(Vec3::new(
                                        x - 2.0,
                                        0.0,
                                        y,
                                    ))
                                    .with_rotation(Quat::from_rotation_y(FRAC_PI_2))
                                    .with_scale(wall_scale),
                                    ..default()
                                },
                                RigidBody::Static,
                                Collider::cuboid(4.0, 40.0, 0.2),
                            ));
                        }
                        if !flag.contains(Flags::RIGHT) {
                            parent.spawn((
                                SceneBundle {
                                    scene: assets.wall.clone(),
                                    transform: Transform::from_translation(Vec3::new(
                                        x + 2.0,
                                        0.0,
                                        y,
                                    ))
                                    .with_rotation(Quat::from_rotation_y(FRAC_PI_2))
                                    .with_scale(wall_scale),
                                    ..default()
                                },
                                RigidBody::Static,
                                Collider::cuboid(4.0, 40.0, 0.2),
                            ));
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
                            parent.spawn((PointLightBundle {
                                transform: Transform::from_translation(Vec3::new(x, 5.0, y)),
                                point_light: PointLight {
                                    intensity: 1_500_000.0,
                                    shadows_enabled: true,
                                    range: 20.0,
                                    ..default()
                                },

                                ..default()
                            },));
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
                        Tile::In => {
                            parent.spawn((
                                SceneBundle {
                                    scene: assets.traps_grate.clone(),
                                    transform: Transform::from_translation(Vec3::new(x, 0.0, y)),
                                    ..default()
                                },
                                RigidBody::Static,
                                Collider::cuboid(4.0, 0.2, 4.0),
                            ));
                            parent.spawn(PbrBundle {
                                transform: Transform::from_translation(Vec3::new(x, -0.1, y))
                                    .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                                material: assets.in_material.clone(),
                                mesh: assets.undergrate_mesh.clone(),
                                ..default()
                            });
                        }
                        Tile::Out => {
                            parent.spawn((
                                SceneBundle {
                                    scene: assets.traps_grate.clone(),
                                    transform: Transform::from_translation(Vec3::new(x, 0.0, y)),
                                    ..default()
                                },
                                RigidBody::Static,
                                Collider::cuboid(4.0, 0.2, 4.0),
                            ));
                            parent.spawn(PbrBundle {
                                transform: Transform::from_translation(Vec3::new(x, -0.1, y))
                                    .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                                material: assets.out_material.clone(),
                                mesh: assets.undergrate_mesh.clone(),
                                ..default()
                            });
                        }
                        Tile::Chest(direction) => {
                            parent.spawn(PointLightBundle {
                                transform: Transform::from_translation(Vec3::new(x, 5.0, y)),
                                point_light: PointLight {
                                    intensity: 1_000_000.0,
                                    color: palettes::tailwind::YELLOW_800.into(),
                                    shadows_enabled: true,
                                    ..default()
                                },
                                ..default()
                            });
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
                }
            }
        });

    (
        (level.floors[0].len() * 4, level.floors[0][0].len() * 4),
        level.as_navmesh(vec![]),
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
