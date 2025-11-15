use crate::gameplay::{
    player::{aim::*, health::*, movement::*, stamina::*, },
    inventory::{items::*, pickup::*},
};
use super::{camera::*, setup::*, world::*};
use bevy::prelude::*;

pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePos {
            position: Vec2::new(0.0, 0.0),
        })
        .add_message::<PickupMessage>()
        .add_plugins(ItemsPlugin)
        .add_systems(
            Startup,
            (init_camera, init_environment, spawn_player, spawn_objects),
        )
        .add_systems(
            Update,
            (
                update_camera,
                get_mouse_position,
                rotate_aim.after(get_mouse_position),
                movement,
                run,
                apply_velocity,
                update_stamina,
                update_health,
                restore_stamina,
                update_fog,
                apply_fog_visuals,
                draw_visibility,
            ),
        );
    }
}
