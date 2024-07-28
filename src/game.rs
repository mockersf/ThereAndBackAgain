use std::{
    collections::HashSet,
    f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI, TAU},
    time::Duration,
};

use avian3d::{
    collision::{Collider, CollidingEntities},
    prelude::{CollisionLayers, LinearVelocity, LockedAxes, RigidBody},
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

use crate::{
    assets::GameAssets,
    audio::AudioTrigger,
    levels::{AnimatedKind, Level},
    GameState,
};

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
            .add_systems(Update, set_weapons.run_if(resource_exists::<GameAssets>));
    }
}

#[derive(Resource)]
pub struct NavMesh(pub polyanya::Mesh);

#[derive(Resource, PartialEq, Eq)]
pub enum PathStatus {
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
    mut audio_trigger: EventWriter<AudioTrigger>,
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
                    ColliderKind::Hobbit,
                    CollisionLayers::new(0b100, 0b111),
                ))
                .with_children(|p| {
                    p.spawn((
                        SceneBundle {
                            scene: assets.character.clone(),
                            transform: Transform::from_translation(vec3(0.0, -1.0, 0.0)),
                            ..default()
                        },
                        AnimatedKind::Hobbit,
                    ));
                });
            audio_trigger.send(AudioTrigger::Spawn);

            *local_timer = None;
        }
    } else if hobbits.iter().len() < level.0.nb_hobbits as usize {
        let timer = if initial {
            if level.0.message.is_some() {
                Timer::from_seconds(7.5, TimerMode::Once)
            } else {
                Timer::from_seconds(1.5, TimerMode::Once)
            }
        } else {
            Timer::from_seconds(level.0.spawn_delay, TimerMode::Once)
        };
        *local_timer = Some(timer);
    }
}

#[derive(Resource)]
struct WalkAnimations {
    animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    graph: Handle<AnimationGraph>,
}
#[derive(Resource)]
struct AttackAnimations {
    animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    graph: Handle<AnimationGraph>,
}

fn prepare_animations(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    {
        let mut graph = AnimationGraph::new();
        let animations = graph
            .add_clips([assets.character_walk.clone()], 1.0, graph.root)
            .collect();
        let graph = graphs.add(graph);
        commands.insert_resource(WalkAnimations {
            animations,
            graph: graph.clone(),
        });
    }

    {
        let mut graph = AnimationGraph::new();
        let animations = graph
            .add_clips([assets.skeleton_attack.clone()], 1.0, graph.root)
            .collect();
        let graph = graphs.add(graph);
        commands.insert_resource(AttackAnimations {
            animations,
            graph: graph.clone(),
        });
    }
}

fn add_animations(
    mut commands: Commands,
    walk_animations: Res<WalkAnimations>,
    attack_animations: Res<AttackAnimations>,
    mut players: Query<(Entity, &Parent, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parents: Query<&Parent>,
    animated: Query<&AnimatedKind>,
) {
    for (entity, parent, mut player) in &mut players {
        match parents
            .get(parent.get())
            .and_then(|p| animated.get(p.get()))
        {
            Ok(AnimatedKind::Skeleton) => {
                let mut transitions = AnimationTransitions::new();
                transitions
                    .play(&mut player, attack_animations.animations[0], Duration::ZERO)
                    .set_speed(0.7)
                    .repeat();
                commands
                    .entity(entity)
                    .insert(attack_animations.graph.clone())
                    .insert(transitions);
            }
            _ => {
                let mut transitions = AnimationTransitions::new();
                transitions
                    .play(&mut player, walk_animations.animations[0], Duration::ZERO)
                    .repeat();
                commands
                    .entity(entity)
                    .insert(walk_animations.graph.clone())
                    .insert(transitions);
            }
        }
    }
}

fn set_weapons(
    mut commands: Commands,
    mut scenes_loaded: EventReader<SceneInstanceReady>,
    scene_instances: Query<&SceneInstance>,
    names: Query<(Entity, &Name)>,
    scene_spawner: Res<SceneSpawner>,
    animated: Query<&AnimatedKind>,
    assets: Res<GameAssets>,
) {
    for scene in scenes_loaded.read() {
        match animated.get(scene.parent) {
            Ok(AnimatedKind::Skeleton) => {
                let arm_name = Name::new("hand.r");
                let scene_instance = scene_instances.get(scene.parent).unwrap();
                scene_spawner
                    .iter_instance_entities(**scene_instance)
                    .for_each(|e| {
                        if let Ok((entity, name)) = names.get(e) {
                            if name == &arm_name {
                                commands.entity(entity).with_children(|p| {
                                    p.spawn((SceneBundle {
                                        scene: assets.skeleton_sword.clone(),
                                        transform: Transform::from_rotation(
                                            Quat::from_rotation_y(-PI)
                                                * Quat::from_rotation_z(-FRAC_PI_2),
                                        )
                                        .with_scale(Vec3::splat(1.15)),
                                        ..default()
                                    },))
                                        .with_children(|p| {
                                            p.spawn((
                                                SpatialBundle::from_transform(
                                                    Transform::from_translation(Vec3::new(
                                                        0.0, 0.4, 0.0,
                                                    )),
                                                ),
                                                RigidBody::Static,
                                                LockedAxes::ALL_LOCKED,
                                                Collider::cuboid(0.7, 2.3, 0.2),
                                                ColliderKind::Blade,
                                                CollisionLayers::new(0b001, 0b100),
                                            ));
                                        });
                                });
                            }
                        }
                    });
            }
            Ok(AnimatedKind::Hobbit) => {
                let weapon_names = [
                    Name::new("Knife"),
                    Name::new("Knife_Offhand"),
                    Name::new("1H_Crossbow"),
                    Name::new("2H_Crossbow"),
                    Name::new("Throwable"),
                ];
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
            _ => {}
        }
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
    mut audio_trigger: EventWriter<AudioTrigger>,
) {
    for (entity, mut target, transform, mut hobbit) in &mut bodies {
        if target.path.is_empty() {
            if matches!(hobbit.state, HobbitState::Tired)
                && transform.translation.distance(target.next) < 1.5
            {
                game_events.send(GameEvent::HomeWithTreasure);
                commands.entity(entity).despawn_recursive();
                audio_trigger.send(AudioTrigger::Home);
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
                audio_trigger.send(AudioTrigger::Treasure);
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
                reevaluate: Timer::from_seconds(0.5, TimerMode::Repeating),
            });
            path_status.set_if_neq(PathStatus::Open);
        } else {
            warn!("path blocked");
            path_status.set_if_neq(PathStatus::Blocked);
            *local_timer = Some(Timer::from_seconds(0.5, TimerMode::Once));
        }
    }
}

fn reevaluate_path(
    mut commands: Commands,
    level: Res<ActiveLevel>,
    mut bodies: Query<(Entity, &Hobbit, &Transform, &mut Target)>,
    mut navmesh: ResMut<NavMesh>,
    time: Res<Time>,
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
                entity_deltas.remove(&entity);
            } else {
                warn!("path blocked on recompute");
                let delta = entity_deltas.entry(entity).or_insert(0.1);
                *delta *= 3.0;
                if *delta > 10.0 {
                    commands.entity(entity).remove::<Target>();
                }
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

#[derive(Component)]
struct Explosion(Timer);

#[derive(Component, PartialEq, Eq, Debug)]
pub enum ColliderKind {
    Hobbit,
    Blade,
}

fn colliding_hobbits(
    mut commands: Commands,
    query: Query<(
        Entity,
        &CollidingEntities,
        Option<&Hobbit>,
        &Transform,
        &ColliderKind,
    )>,
    mut game_events: EventWriter<GameEvent>,
    mut explosion_query: Query<(Entity, &mut Explosion)>,
    time: Res<Time>,
    mut audio_trigger: EventWriter<AudioTrigger>,
) {
    for (entity, colliding_entities, hobbit, transform, _) in &query {
        let Some(hobbit) = hobbit else {
            continue;
        };
        for other_entity in colliding_entities.iter() {
            if let Ok((_, _, other_hobbit, _, other_kind)) = query.get(*other_entity) {
                if other_kind == &ColliderKind::Blade
                    || (other_hobbit.is_some() && other_hobbit.unwrap().state != hobbit.state)
                {
                    audio_trigger.send(AudioTrigger::Hurt);

                    game_events.send(GameEvent::CollidedWithHobbit);
                    commands.entity(entity).despawn_recursive();
                    commands
                        .spawn(ParticleSpawnerBundle::from_settings(
                            ParticleSpawnerSettings {
                                one_shot: true,
                                rate: 500.0,
                                emission_shape: EmissionShape::Circle {
                                    normal: Vec3::Y,
                                    radius: 1.0,
                                },
                                lifetime: RandF32::constant(0.4),
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
                                color: Gradient::constant(palettes::tailwind::RED_500.into()),
                                blend_mode: BlendMode::Blend,
                                linear_drag: 0.1,
                                pbr: false,
                                ..default()
                            },
                        ))
                        .insert((
                            *transform,
                            Explosion(Timer::from_seconds(0.5, TimerMode::Once)),
                        ));
                }
            }
        }
    }

    for (entity, mut explosion) in &mut explosion_query {
        if explosion.0.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
