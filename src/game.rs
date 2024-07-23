use std::{f32::consts::PI, time::Duration};

use avian3d::{
    collision::Collider,
    prelude::{Friction, LinearVelocity, LockedAxes, RigidBody},
};
use bevy::{
    math::vec3,
    prelude::*,
    scene::{SceneInstance, SceneInstanceReady},
};

use crate::{assets::GameAssets, levels::Level, GameState};

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), prepare_animations)
            .add_systems(
                Update,
                (
                    spawn_hobbits,
                    add_animations,
                    move_to_target,
                    reach_target,
                    give_target,
                )
                    .run_if(resource_exists::<ActiveLevel>),
            )
            .add_systems(Update, remove_weapons);
    }
}

#[derive(Resource)]
pub struct NavMesh(pub vleue_navigator::NavMesh);

#[derive(Resource)]
pub struct ActiveLevel(pub Level);

enum HobbitState {
    LFG,
    Tired,
}

#[derive(Component)]
struct Hobbit {
    state: HobbitState,
}

#[derive(Component)]
struct Target(Vec3);

fn spawn_hobbits(
    mut commands: Commands,
    hobbits: Query<&Hobbit>,
    time: Res<Time>,
    level: Res<ActiveLevel>,
    mut local_timer: Local<Option<Timer>>,
    assets: Res<GameAssets>,
) {
    if level.is_added() || level.is_changed() {
        *local_timer = None;
    }
    if let Some(timer) = local_timer.as_mut() {
        if timer.tick(time.delta()).just_finished() {
            commands
                .spawn((
                    SpatialBundle::from_transform(Transform::from_translation(Vec3::new(
                        level.0.start.1 as f32 * 4.0,
                        2.0,
                        level.0.start.2 as f32 * 4.0,
                    ))),
                    RigidBody::Dynamic,
                    LockedAxes::new().lock_rotation_x().lock_rotation_z(),
                    Collider::capsule(0.5, 1.0),
                    Target(Vec3::new(
                        level.0.end.1 as f32 * 4.0,
                        2.0,
                        level.0.end.2 as f32 * 4.0,
                    )),
                    Friction::new(0.0),
                    Hobbit {
                        state: HobbitState::LFG,
                    },
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
    } else {
        if hobbits.iter().len() < level.0.nb_hobbits as usize {
            *local_timer = Some(Timer::from_seconds(level.0.spawn_delay, TimerMode::Once));
        }
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

fn move_to_target(
    time: Res<Time>,
    mut bodies: Query<(Entity, &mut LinearVelocity, &Target, &mut Transform)>,
) {
    let delta_time = time.delta_seconds();

    for (_, mut linvel, target, mut transform) in &mut bodies {
        let mut direction = target.0 - transform.translation;
        direction = direction.normalize() * 2.0;
        linvel.x += direction.x * delta_time;
        linvel.z += direction.z * delta_time;
        transform.rotation = Quat::from_rotation_y(-direction.x.atan2(direction.z) + PI);
    }
}

fn reach_target(
    mut commands: Commands,
    mut bodies: Query<(Entity, &Target, &Transform, &mut Hobbit)>,
) {
    for (entity, target, transform, mut hobbit) in &mut bodies {
        if transform.translation.distance(target.0) < 2.0 {
            if matches!(hobbit.state, HobbitState::LFG) {
                hobbit.state = HobbitState::Tired;
                commands.entity(entity).remove::<Target>();
            } else {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn give_target(
    mut commands: Commands,
    level: Res<ActiveLevel>,
    bodies: Query<(Entity, &Hobbit), Without<Target>>,
) {
    for (entity, hobbit) in &bodies {
        match hobbit.state {
            HobbitState::LFG => {
                commands.entity(entity).insert(Target(Vec3::new(
                    level.0.end.1 as f32 * 4.0,
                    0.0,
                    level.0.end.2 as f32 * 4.0,
                )));
            }
            HobbitState::Tired => {
                commands.entity(entity).insert(Target(Vec3::new(
                    level.0.start.1 as f32 * 4.0,
                    0.0,
                    level.0.start.2 as f32 * 4.0,
                )));
            }
        }
    }
}
