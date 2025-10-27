use super::{
    player::player::PlayerPlugin,
    world::world::WorldPlugin,
}; 
use bevy::prelude::*;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WorldPlugin, PlayerPlugin));
    }
}
