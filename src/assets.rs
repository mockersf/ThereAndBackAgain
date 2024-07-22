use bevy::prelude::*;

use crate::levels::Level;

#[derive(Resource)]
#[allow(dead_code)]
pub struct GameAssets {
    pub character: Handle<Scene>,
    pub items_warrior: Handle<Scene>,
    pub items_mage: Handle<Scene>,
    pub items_obstacle: Handle<Scene>,
    pub traps_warrior: Handle<Scene>,
    pub traps_mage: Handle<Scene>,
    pub traps_spike: Handle<Scene>,
    pub traps_grate: Handle<Scene>,
    pub floor: Handle<Scene>,
    pub chest: Handle<Scene>,
    pub coin_stack: Handle<Scene>,
    pub levels: Vec<Handle<Level>>,
    pub wall: Handle<Scene>,
    pub wall_corner: Handle<Scene>,
}
