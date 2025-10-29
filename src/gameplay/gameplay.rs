use bevy::prelude::*; 
use super::{
    aim::*,
    camera::*,
    setup::*,
    world::*,
};

pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (init_camera, init_environment, spawn_player))
            .add_systems(Update, (update_camera, get_mouse_position)); 
    }
}