use bevy::prelude::*;

#[derive(Resource)]
#[allow(dead_code)]
pub struct Assets {
    pub character: Handle<Gltf>,
    pub items_warrior: Handle<Gltf>,
    pub items_mage: Handle<Gltf>,
    pub items_obstacle: Handle<Gltf>,
    pub traps_warrior: Handle<Gltf>,
    pub traps_mage: Handle<Gltf>,
    pub traps_spike: Handle<Gltf>,
    pub traps_grate: Handle<Gltf>,
}
