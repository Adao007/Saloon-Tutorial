use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use gameplay::gameplay::GameplayPlugin;

mod gameplay;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TilemapPlugin, GameplayPlugin))
        .run();
}
