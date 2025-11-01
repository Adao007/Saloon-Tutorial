use super::{aim::MousePos, player::Player};
use bevy::prelude::*;

pub fn walk(
    player_query: Single<&mut Transform, With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut transform = player_query.into_inner();

    if keyboard_input.pressed(KeyCode::KeyW) {
        transform.translation.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        transform.translation.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        transform.translation.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        transform.translation.x += 1.0;
    }
}
