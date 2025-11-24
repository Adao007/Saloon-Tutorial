use crate::gameplay::{
    player::{aim::*, health::*, movement::*, stamina::*, },
    inventory::{inventory::*, items::*, pickup::*,},
};
use super::{camera::*, setup::*, world::*};
use bevy::prelude::*;

pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(MousePos {
            position: Vec2::new(0.0, 0.0),
        })
        .add_plugins(InventoryPlugin)
        .add_message::<PickupMessage>()
        .add_systems(
            Startup,
            (init_camera, init_environment, init_litter_text, spawn_player, spawn_objects),
        )
        .add_systems(
            Update,
            (
                update_camera,
                get_mouse_position,
                move_cursor_text.after(get_mouse_position),
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
