use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use gameplay::gameplay::GameplayPlugin;

mod gameplay;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins, 
            PhysicsPlugins::default().with_length_unit(50.0), 
            TilemapPlugin, 
            GameplayPlugin
        ))
        .run();
}