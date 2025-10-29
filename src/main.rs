use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
pub mod game;
pub mod player;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TilemapPlugin, game::game::GamePlugin))
        .run();
}
