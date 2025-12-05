use bevy::prelude::*;
use crate::gameplay::player::aim::MousePos;
use crate::gameplay::player::setup::spawn_player;
use crate::gameplay::player::aim::get_mouse_position;
use crate::gameplay::player::aim::rotate_aim;
use crate::gameplay::player::movement::movement;
use crate::gameplay::player::movement::run;
use crate::gameplay::player::movement::apply_velocity;
use crate::gameplay::player::stamina::update_stamina;
use crate::gameplay::player::health::update_health;
use crate::gameplay::player::stamina::restore_stamina;

pub struct PlayerPlugin; 
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MousePos {
                position: Vec2::new(0.0, 0.0),
            })
            .add_systems(
            Startup,
            spawn_player,
            )
            .add_systems(
            Update,
            (
                get_mouse_position,
                rotate_aim.after(get_mouse_position),
                movement,
                run,
                apply_velocity,
                update_stamina,
                update_health,
                restore_stamina,
            ));
    }
}

#[derive(Component)]
pub struct Player{
    pub speed: f32, 
} 