use super::{aim::*, camera::*, movement::*, setup::*, world::*};
use bevy::prelude::*;

pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePos {
            position: Vec2::new(0.0, 0.0),
        })
        .add_systems(Startup, (init_camera, init_environment, spawn_player))
        .add_systems(
            Update,
            (
                update_camera,
                get_mouse_position,
                rotate_aim.after(get_mouse_position),
                walk,
            ),
        );
    }
}
