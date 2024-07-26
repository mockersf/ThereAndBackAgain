use std::{
    collections::HashSet,
    f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU},
    time::Duration,
};

use avian3d::{
    collision::{Collider, CollidingEntities},
    prelude::{LinearVelocity, LockedAxes, RigidBody},
};
use bevy::{
    color::palettes,
    ecs::entity::EntityHashMap,
    math::{vec2, vec3},
    prelude::*,
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

use crate::{assets::GameAssets, levels::Level, GameState};

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PathStatus::Open)
            .add_event::<GameEvent>()
            .add_systems(OnExit(GameState::Loading), prepare_animations)
            .add_systems(
                PreUpdate,
                colliding_hobbits.run_if(resource_exists::<ActiveLevel>),
            )
            .add_systems(
                Update,
                (
                    spawn_hobbits,
                    add_animations,
                    move_to_target,
                    reach_target,
                    give_target,
                    reevaluate_path,
                    #[cfg(feature = "debug")]
                    display_paths,
                )
                    .run_if(resource_exists::<ActiveLevel>),
            )
            .add_systems(Update, remove_weapons);
    }
}

#[derive(Resource)]
pub struct NavMesh(pub polyanya::Mesh);

#[derive(Resource)]
enum PathStatus {
    Open,
    Blocked,
}

#[derive(Resource)]
pub struct ActiveLevel(pub Level);

#[derive(Clone, Copy, PartialEq, Eq)]
enum HobbitState {
    #[allow(clippy::upper_case_acronyms)]
    LFG,
    Tired,
}

#[derive(Component)]
pub struct Hobbit {
    state: HobbitState,
}

#[derive(Component)]
struct Target {
    next: Vec3,
    path: Vec<Vec2>,
    reevaluate: Timer,
}

#[allow(clippy::too_many_arguments)]
fn spawn_hobbits(
    mut commands: Commands,
    hobbits: Query<&Hobbit>,
    time: Res<Time>,
    level: Res<ActiveLevel>,
    mut local_timer: Local<Option<Timer>>,
    assets: Res<GameAssets>,
    state: Res<State<GameState>>,
    mut path_status: ResMut<PathStatus>,
) {
    let mut initial = false;
    if level.is_added() || level.is_changed() {
        initial = true;
        *local_timer = None;
        *path_status = PathStatus::Open;
    }
    if matches!(*path_status, PathStatus::Blocked) {
        return;
    }
    if let Some(timer) = local_timer.as_mut() {
        if timer.tick(time.delta()).just_finished() {
            commands
                .spawn((
                    SpatialBundle::from_transform(Transform::from_translation(Vec3::new(
                        level.0.start.1 as f32 * 4.0,
                        1.2,
                        level.0.start.2 as f32 * 4.0,
                    ))),
                    RigidBody::Dynamic,
                    LockedAxes::new().lock_rotation_x().lock_rotation_z(),
                    Collider::capsule(0.5, 1.0),
                    Hobbit {
                        state: HobbitState::LFG,
                    },
                    StateScoped(*state.get()),
                ))
                .with_children(|p| {
                    p.spawn(SceneBundle {
                        scene: assets.character.clone(),
                        transform: Transform::from_translation(vec3(0.0, -1.0, 0.0)),
                        ..default()
                    });
                });
            *local_timer = None;
        }
    } else if hobbits.iter().len() < level.0.nb_hobbits as usize {
        let mut timer = Timer::from_seconds(level.0.spawn_delay, TimerMode::Once);
        if initial {
            timer.set_elapsed(Duration::from_secs_f32(level.0.spawn_delay * 0.5));
        }
        *local_timer = Some(timer);
    }
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    graph: Handle<AnimationGraph>,
}

fn prepare_animations(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let mut graph = AnimationGraph::new();
    let animations = graph
        .add_clips([assets.character_walk.clone()], 1.0, graph.root)
        .collect();

    // Insert a resource with the current scene information
    let graph = graphs.add(graph);
    commands.insert_resource(Animations {
        animations,
        graph: graph.clone(),
    });
}

fn add_animations(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

fn remove_weapons(
    mut commands: Commands,
    mut scenes_loaded: EventReader<SceneInstanceReady>,
    scene_instances: Query<&SceneInstance>,
    names: Query<(Entity, &Name)>,
    scene_spawner: Res<SceneSpawner>,
) {
    let weapon_names = [
        Name::new("Knife"),
        Name::new("Knife_Offhand"),
        Name::new("1H_Crossbow"),
        Name::new("2H_Crossbow"),
        Name::new("Throwable"),
    ];
    for scene in scenes_loaded.read() {
        let scene_instance = scene_instances.get(scene.parent).unwrap();
        scene_spawner
            .iter_instance_entities(**scene_instance)
            .for_each(|e| {
                if let Ok((entity, name)) = names.get(e) {
                    if weapon_names.contains(name) {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            });
    }
}
const MAX_SPEED: f32 = 8.0;

fn move_to_target(
    time: Res<Time>,
    mut bodies: Query<(Entity, &mut LinearVelocity, &Target, &mut Transform)>,
) {
    let delta_time = time.delta_seconds();

    for (_, mut linvel, target, mut transform) in &mut bodies {
        let full_direction = target.next - transform.translation;
        let desired_velocity = full_direction.xz().normalize() * MAX_SPEED;
        let steering = desired_velocity - linvel.0.xz();
        linvel.x += steering.x * delta_time;
        linvel.z += steering.y * delta_time;
        if linvel.length() > MAX_SPEED {
            linvel.0 = linvel.normalize() * MAX_SPEED;
        }
        if target.path.is_empty() && linvel.length() > full_direction.length() {
            linvel.0 *= 0.9;
        }
        let mut new_rotation = -linvel.0.z.atan2(linvel.0.x) + FRAC_PI_2;
        if new_rotation > PI {
            new_rotation -= TAU;
        }
        transform.rotation = Quat::from_rotation_y(new_rotation);
    }
}

fn reach_target(
    mut commands: Commands,
    mut bodies: Query<(Entity, &mut Target, &Transform, &mut Hobbit)>,
    mut game_events: EventWriter<GameEvent>,
) {
    for (entity, mut target, transform, mut hobbit) in &mut bodies {
        if target.path.is_empty() {
            if matches!(hobbit.state, HobbitState::Tired)
                && transform.translation.distance(target.next) < 1.5
            {
                game_events.send(GameEvent::HomeWithTreasure);
                commands.entity(entity).despawn_recursive();
            }

            if matches!(hobbit.state, HobbitState::LFG)
                && transform.translation.distance(target.next) < 1.0
            {
                hobbit.state = HobbitState::Tired;
                commands.entity(entity).remove::<Target>();
                commands.entity(entity).with_children(|parent| {
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
                                (palettes::tailwind::YELLOW_800 * 5.0).into(),
                            ),
                            blend_mode: BlendMode::Blend,
                            linear_drag: 0.1,
                            pbr: true,
                            ..default()
                        },
                    ));
                });
            }
        } else if !target.path.is_empty()
            && transform.translation.distance(target.next) < MAX_SPEED / 10.0
        {
            let next = target.path.pop().unwrap();
            target.next = vec3(next.x, 1.0, next.y);
        }
    }
}

fn give_target(
    mut commands: Commands,
    level: Res<ActiveLevel>,
    bodies: Query<(Entity, &Hobbit, &Transform), Without<Target>>,
    navmesh: Res<NavMesh>,
    mut path_status: ResMut<PathStatus>,
    mut local_timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    if let Some(timer) = local_timer.as_mut() {
        if timer.tick(time.delta()).just_finished() {
            *local_timer = None;
        } else {
            return;
        }
    }
    for (entity, hobbit, transform) in &bodies {
        let from = vec2(transform.translation.x, transform.translation.z);
        let (to, exclusion) = match hobbit.state {
            HobbitState::LFG => {
                let mut exclusion = HashSet::new();
                exclusion.insert(2);
                (
                    Vec2::new(level.0.end.1 as f32 * 4.0, level.0.end.2 as f32 * 4.0),
                    exclusion,
                )
            }

            HobbitState::Tired => {
                let mut exclusion = HashSet::new();
                exclusion.insert(1);
                (
                    Vec2::new(level.0.start.1 as f32 * 4.0, level.0.start.2 as f32 * 4.0),
                    exclusion,
                )
            }
        };
        if let Some(path) = navmesh.0.path_on_layers(from, to, exclusion) {
            let (next, remaining) = path.path.split_first().unwrap();
            let mut remaining = remaining.to_vec();
            remaining.reverse();
            commands.entity(entity).insert(Target {
                next: vec3(next.x, 1.0, next.y),
                path: remaining,
                reevaluate: Timer::from_seconds(2.0, TimerMode::Repeating),
            });
            *path_status = PathStatus::Open;
        } else {
            warn!("path blocked");
            *path_status = PathStatus::Blocked;
            *local_timer = Some(Timer::from_seconds(0.5, TimerMode::Once));
        }
    }
}

fn reevaluate_path(
    level: Res<ActiveLevel>,
    mut bodies: Query<(Entity, &Hobbit, &Transform, &mut Target)>,
    mut navmesh: ResMut<NavMesh>,
    time: Res<Time>,
    mut path_status: ResMut<PathStatus>,
    mut local_timer: Local<Option<Timer>>,
    mut entity_deltas: Local<EntityHashMap<f32>>,
) {
    if let Some(timer) = local_timer.as_mut() {
        if timer.tick(time.delta()).just_finished() {
            *local_timer = None;
        } else {
            return;
        }
    }
    let mut i = 0;
    for (entity, hobbit, transform, mut target) in &mut bodies {
        if target.reevaluate.tick(time.delta()).finished() {
            let from = vec2(transform.translation.x, transform.translation.z);
            let (to, exclusion) = match hobbit.state {
                HobbitState::LFG => {
                    let mut exclusion = HashSet::new();
                    exclusion.insert(2);
                    (
                        Vec2::new(level.0.end.1 as f32 * 4.0, level.0.end.2 as f32 * 4.0),
                        exclusion,
                    )
                }

                HobbitState::Tired => {
                    let mut exclusion = HashSet::new();
                    exclusion.insert(1);
                    (
                        Vec2::new(level.0.start.1 as f32 * 4.0, level.0.start.2 as f32 * 4.0),
                        exclusion,
                    )
                }
            };
            let entity_delta = entity_deltas.get(&entity).cloned().unwrap_or(0.1);
            navmesh.0.set_delta(entity_delta);
            if let Some(path) = navmesh.0.path_on_layers(from, to, exclusion) {
                i += 1;
                let (next, remaining) = path.path.split_first().unwrap();
                let mut remaining = remaining.to_vec();
                remaining.reverse();
                target.next = vec3(next.x, 1.0, next.y);
                target.path = remaining;
                target.reevaluate.reset();
                *path_status = PathStatus::Open;
                entity_deltas.remove(&entity);
            } else {
                warn!("path blocked on recompute");
                *entity_deltas.entry(entity).or_insert(0.1) *= 3.0;
                *local_timer = Some(Timer::from_seconds(0.25, TimerMode::Once));
            }
            navmesh.0.set_delta(0.1);
        }
    }
    if i != 0 {
        info!("re-evaluating path for {} hobbits", i);
    }
}

#[cfg(feature = "debug")]
fn display_paths(query: Query<(&Transform, &Target)>, mut gizmos: Gizmos) {
    use bevy::color::palettes;

    for (transform, target) in &query {
        let mut path = target
            .path
            .iter()
            .map(|v| vec3(v.x, 0.2, v.y))
            .collect::<Vec<_>>();
        path.push(vec3(target.next.x, 0.3, target.next.z));
        path.push(vec3(transform.translation.x, 0.3, transform.translation.z));
        gizmos.linestrip(path, palettes::tailwind::TEAL_300);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Event)]
pub enum GameEvent {
    HomeWithTreasure,
    CollidedWithHobbit,
}

fn colliding_hobbits(
    mut commands: Commands,
    query: Query<(Entity, &CollidingEntities, &Hobbit)>,
    mut game_events: EventWriter<GameEvent>,
) {
    for (entity, colliding_entities, hobbit) in &query {
        for other_entity in colliding_entities.iter() {
            if let Ok((_, _, other_hobbit)) = query.get(*other_entity) {
                if other_hobbit.state != hobbit.state {
                    game_events.send(GameEvent::CollidedWithHobbit);
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}
