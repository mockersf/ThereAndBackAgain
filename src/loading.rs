use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
};

use bevy::{asset::LoadedFolder, color::palettes, prelude::*, tasks::AsyncComputeTaskPool};
use event_listener::Event;

use rand::Rng;

use crate::{
    assets::{GameAssets, RawGameAssets},
    levels::Level,
    GameState,
};

const CURRENT_STATE: GameState = GameState::Loading;

#[derive(Resource)]
struct Screen {
    done: Timer,
}

const NB_LEVELS: usize = 12;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Screen {
            #[cfg(feature = "release")]
            done: Timer::from_seconds(1.0, TimerMode::Once),
            #[cfg(not(feature = "release"))]
            done: Timer::from_seconds(0.1, TimerMode::Once),
        })
        .add_systems(OnEnter(CURRENT_STATE), setup)
        .add_systems(Update, (done, animate_logo).run_if(in_state(CURRENT_STATE)));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading screen");
    let vleue_logo = asset_server.load("embedded://there_and_back_again/branding/logo.png");
    let bevy_logo =
        asset_server.load("embedded://there_and_back_again/branding/bevy_logo_dark.png");
    let birdoggo_logo = asset_server.load("embedded://there_and_back_again/branding/birdoggo.png");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
            StateScoped(CURRENT_STATE),
        ))
        .with_children(|commands| {
            commands.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Px(300.0),
                        height: Val::Auto,
                        margin: UiRect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    image: UiImage {
                        texture: vleue_logo,
                        ..default()
                    },
                    ..default()
                },
                SplashGiggle(Timer::from_seconds(0.05, TimerMode::Repeating)),
            ));
            commands.spawn(ImageBundle {
                style: Style {
                    right: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                    position_type: PositionType::Absolute,
                    width: Val::Auto,
                    height: Val::Px(50.0),
                    ..default()
                },
                image: UiImage {
                    texture: bevy_logo,
                    ..default()
                },
                ..default()
            });
            commands.spawn(ImageBundle {
                style: Style {
                    right: Val::Px(10.0),
                    bottom: Val::Px(70.0),
                    position_type: PositionType::Absolute,
                    width: Val::Auto,
                    height: Val::Px(50.0),
                    ..default()
                },
                image: UiImage {
                    texture: birdoggo_logo,
                    ..default()
                },
                ..default()
            });
        });

    let (barrier, guard) = AssetBarrier::new();
    commands.insert_resource(RawGameAssets {
        #[cfg(not(target_arch = "wasm32"))]
        levels: asset_server.load_folder("levels"),
        #[cfg(target_arch = "wasm32")]
        levels: (0..=NB_LEVELS)
            .map(|i| asset_server.load_acquire(format!("levels/{:0>2}.level", i), guard.clone()))
            .collect(),
        character: asset_server.load_acquire("characters/Rogue.glb", guard.clone()),
        traps_grate: asset_server.load_acquire(
            GltfAssetLabel::Scene(0).from_asset("ground/floor_tile_big_grate_open.gltf"),
            guard.clone(),
        ),
        floor: asset_server.load_acquire(
            GltfAssetLabel::Scene(0).from_asset("ground/floor_tile_large.gltf"),
            guard.clone(),
        ),
        chest: asset_server.load_acquire(
            GltfAssetLabel::Scene(0).from_asset("treasure/chest_gold.gltf"),
            guard.clone(),
        ),
        coin_stack: asset_server.load_acquire(
            GltfAssetLabel::Scene(0).from_asset("treasure/coin_stack_large.gltf"),
            guard.clone(),
        ),
        wall: asset_server.load_acquire(
            GltfAssetLabel::Scene(0).from_asset("scenery/wall.gltf"),
            guard.clone(),
        ),
        wall_corner: asset_server.load_acquire(
            GltfAssetLabel::Scene(0).from_asset("scenery/wall_corner.gltf"),
            guard.clone(),
        ),
        obstacle: asset_server.load_acquire(
            GltfAssetLabel::Scene(0).from_asset("items/crates_stacked.gltf"),
            guard.clone(),
        ),
        icon_obstacle: asset_server.load_acquire("icons/obstacle.png", guard.clone()),
        skeleton: asset_server.load_acquire("traps/Skeleton_Warrior.glb", guard.clone()),
        skeleton_sword: asset_server.load_acquire(
            GltfAssetLabel::Scene(0).from_asset("traps/Skeleton_Blade.gltf"),
            guard.clone(),
        ),
    });
    let future = barrier.wait_async();
    commands.insert_resource(barrier);

    let loading_state = Arc::new(AtomicBool::new(false));
    commands.insert_resource(AsyncLoadingState(loading_state.clone()));

    // await the `AssetBarrierFuture`.
    AsyncComputeTaskPool::get()
        .spawn(async move {
            future.await;
            // Notify via `AsyncLoadingState`
            loading_state.store(true, Ordering::Release);
        })
        .detach();
}

#[derive(Component)]
struct SplashGiggle(Timer);

#[allow(clippy::too_many_arguments)]
fn done(
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    folders: Res<Assets<LoadedFolder>>,
    levels: Res<Assets<Level>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    raw_assets: Res<RawGameAssets>,
    time: Res<Time>,
    mut screen: ResMut<Screen>,
    mut state: ResMut<NextState<GameState>>,
    loading_state: Res<AsyncLoadingState>,
    mut asset_ready: Local<bool>,
) {
    if !*asset_ready && loading_state.0.load(Ordering::Acquire) {
        let mut loaded_levels;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(folder) = folders.get(&raw_assets.levels) else {
                return;
            };
            loaded_levels = folder
                .handles
                .iter()
                .map(|h| h.clone().typed())
                .collect::<Vec<_>>();
            loaded_levels.sort_by_key(|h| &levels.get(h).unwrap().file);
            info!("loaded {} levels", folder.handles.len());
            assert_eq!(loaded_levels.len(), NB_LEVELS + 1);
        }
        #[cfg(target_arch = "wasm32")]
        {
            loaded_levels = raw_assets.levels.clone();
        }

        *asset_ready = true;
        let character = gltfs.get(&raw_assets.character).unwrap();
        let skeleton = gltfs.get(&raw_assets.skeleton).unwrap();

        commands.insert_resource(GameAssets {
            character: character.scenes[0].clone(),
            character_walk: character.named_animations.get("Walking_A").unwrap().clone(),
            skeleton: skeleton.scenes[0].clone(),
            skeleton_attack: skeleton
                .named_animations
                .get("2H_Melee_Attack_Spinning")
                .unwrap()
                .clone(),
            traps_grate: raw_assets.traps_grate.clone(),
            floor: raw_assets.floor.clone(),
            chest: raw_assets.chest.clone(),
            coin_stack: raw_assets.coin_stack.clone(),
            levels: loaded_levels,
            wall: raw_assets.wall.clone(),
            wall_corner: raw_assets.wall_corner.clone(),
            in_material: materials.add(StandardMaterial {
                base_color: palettes::tailwind::GREEN_500.into(),
                emissive: (palettes::tailwind::GREEN_700 * 6.0).into(),
                ..default()
            }),
            out_material: materials.add(StandardMaterial {
                base_color: palettes::tailwind::RED_500.into(),
                emissive: (palettes::tailwind::RED_900 * 6.0).into(),
                ..default()
            }),
            one_way_material: materials.add(StandardMaterial {
                base_color: palettes::tailwind::BLUE_500.into(),
                emissive: (palettes::tailwind::BLUE_900 * 3.0).into(),
                ..default()
            }),
            undergrate_mesh: meshes.add(Rectangle::new(4.0, 4.0).mesh()),
            obstacle: raw_assets.obstacle.clone(),
            icon_obstacle: raw_assets.icon_obstacle.clone(),
            skeleton_sword: raw_assets.skeleton_sword.clone(),
        })
    }
    if screen.done.tick(time.delta()).finished() && *asset_ready {
        #[cfg(not(feature = "builder"))]
        state.set(GameState::Menu);
        #[cfg(feature = "builder")]
        state.set(GameState::InGame);
    }
}

fn animate_logo(time: Res<Time>, mut query: Query<(&mut SplashGiggle, &mut Transform)>) {
    for (mut timer, mut transform) in query.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            let scale = transform.scale;
            if (scale.x - 1.) > 0.01 {
                *transform = Transform::IDENTITY;
                continue;
            }

            let mut rng = rand::thread_rng();
            let act = rng.gen_range(0..100);
            if act > 50 {
                let scale_diff = 0.02;
                let new_scale: f32 = rng.gen_range((1. - scale_diff)..(1. + scale_diff));
                *transform = Transform::from_scale(Vec3::splat(new_scale));
            }
        }
    }
}

#[derive(Debug, Resource, Deref)]
struct AssetBarrier(Arc<AssetBarrierInner>);

#[derive(Debug, Deref)]
struct AssetBarrierGuard(Arc<AssetBarrierInner>);

#[derive(Debug, Resource)]
struct AssetBarrierInner {
    count: AtomicU32,
    notify: Event,
}

#[derive(Debug, Resource)]
struct AsyncLoadingState(Arc<AtomicBool>);

impl AssetBarrier {
    fn new() -> (AssetBarrier, AssetBarrierGuard) {
        let inner = Arc::new(AssetBarrierInner {
            count: AtomicU32::new(1),
            notify: Event::new(),
        });
        (AssetBarrier(inner.clone()), AssetBarrierGuard(inner))
    }

    fn wait_async(&self) -> impl Future<Output = ()> + 'static {
        let shared = self.0.clone();
        async move {
            loop {
                let listener = shared.notify.listen();
                if shared.count.load(Ordering::Acquire) == 0 {
                    return;
                }
                listener.await;
            }
        }
    }
}

impl Clone for AssetBarrierGuard {
    fn clone(&self) -> Self {
        self.count.fetch_add(1, Ordering::AcqRel);
        AssetBarrierGuard(self.0.clone())
    }
}

impl Drop for AssetBarrierGuard {
    fn drop(&mut self) {
        let prev = self.count.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            self.notify.notify(usize::MAX);
        }
    }
}
