#[cfg(not(target_arch = "wasm32"))]
use bevy::asset::LoadedFolder;
use bevy::prelude::*;

use crate::levels::Level;

#[derive(Resource)]
pub struct RawGameAssets {
    pub character: Handle<Gltf>,
    pub traps_grate: Handle<Scene>,
    pub floor: Handle<Scene>,
    pub chest: Handle<Scene>,
    pub coin_stack: Handle<Scene>,
    #[cfg(not(target_arch = "wasm32"))]
    pub levels: Handle<LoadedFolder>,
    #[cfg(target_arch = "wasm32")]
    pub levels: Vec<Handle<Level>>,
    pub wall: Handle<Scene>,
    pub wall_corner: Handle<Scene>,
    pub obstacle: Handle<Scene>,
    pub icon_obstacle: Handle<Image>,
}
#[derive(Resource)]
pub struct GameAssets {
    pub character: Handle<Scene>,
    pub character_walk: Handle<AnimationClip>,
    pub traps_grate: Handle<Scene>,
    pub floor: Handle<Scene>,
    pub chest: Handle<Scene>,
    pub coin_stack: Handle<Scene>,
    pub levels: Vec<Handle<Level>>,
    pub wall: Handle<Scene>,
    pub wall_corner: Handle<Scene>,
    pub out_material: Handle<StandardMaterial>,
    pub in_material: Handle<StandardMaterial>,
    pub undergrate_mesh: Handle<Mesh>,
    pub obstacle: Handle<Scene>,
    pub icon_obstacle: Handle<Image>,
}
