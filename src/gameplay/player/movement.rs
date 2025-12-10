use super::{
    stamina::Stamina,
    player::Player,
};
use crate::gameplay::inventory::ui::InventoryUi;
use bevy::prelude::*;

const DIRECTION: f32 = 1.0; 
const RUN_SPEED: f32 = 170.0;
const STAMINA_DRAIN: f32 = 0.5; 
const EMPTY: f32 = 0.0;

#[derive(Component)]
pub struct Velocity {
    pub linvel: Vec3,
}

#[derive(Component)]
pub struct Speed {
    pub base: f32, 
    pub current: f32, 
}

pub fn movement(
    mut query: Query<(&mut Velocity, &Speed), With<Player>>, 
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (mut velocity, speed) in &mut query {
        let mut direction = Vec3::ZERO; 

        if keyboard_input.pressed(KeyCode::KeyW) { direction.y += DIRECTION; }
        if keyboard_input.pressed(KeyCode::KeyS) { direction.y -= DIRECTION; }
        if keyboard_input.pressed(KeyCode::KeyA) { direction.x -= DIRECTION; }
        if keyboard_input.pressed(KeyCode::KeyD) { direction.x += DIRECTION; }

        if direction.length() > 0.0 {
            direction = direction.normalize(); 
            velocity.linvel = direction * speed.current; 
        } 
        else {
            velocity.linvel = Vec3::ZERO; 
        }
    }
}

pub fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity), With<Player>>, 
    time: Res<Time>, 
) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.linvel * time.delta_secs(); 
    }
}

pub fn run(
    player_query: Single<(&mut Stamina, &mut Speed, &Velocity), With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut stamina, mut speed, velocity) = player_query.into_inner(); 

    if keyboard_input.pressed(KeyCode::ShiftLeft) && velocity.linvel != Vec3::ZERO {
        speed.current = RUN_SPEED; 
        if stamina.current > EMPTY { stamina.current -= STAMINA_DRAIN; }
        else { stamina.current = EMPTY; }
    }
    else if keyboard_input.just_released(KeyCode::ShiftLeft) || stamina.current == EMPTY {
        speed.current = speed.base;
    }
}

pub fn prevent_movement (
    inventory: Single<&InventoryUi>,
    mut velocity: Single<&mut Velocity, With<Player>>, 
) {
    if !inventory.activated {
        return; 
    }

    velocity.linvel = Vec3::ZERO;
}