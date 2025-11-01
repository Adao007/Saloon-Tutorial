use super::{
    stamina::Stamina,
    player::Player,
};
use bevy::prelude::*;

const WALK_SPEED: f32 = 1.0; 
const RUN_SPEED: f32 = 1.75;
const STAMINA_DRAIN:f32 = 0.15; 

pub fn movement(
    player_query: Single<(&mut Transform, &Player)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut transform, player) = player_query.into_inner();

    if keyboard_input.pressed(KeyCode::KeyW) {
        transform.translation.y += WALK_SPEED * player.speed;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        transform.translation.y -= WALK_SPEED * player.speed;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        transform.translation.x -= WALK_SPEED * player.speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        transform.translation.x += WALK_SPEED * player.speed;
    }
}

pub fn run(
    player_query: Single<(&mut Stamina, &mut Player)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut stamina, mut player) = player_query.into_inner(); 
    
    if !keyboard_input.pressed(KeyCode::ShiftLeft) || stamina.current <= 0.0 {
        player.speed = 1.0; 
        return; 
    } 
    
    player.speed = RUN_SPEED; 
    stamina.current -= STAMINA_DRAIN;
}