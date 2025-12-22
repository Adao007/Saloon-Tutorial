use avian2d::{math::*, prelude::*}; 
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use serde::{Deserialize, Serialize};
use crate::gameplay::inventory::inventory::Inventory;
use super::{
    player::{Player, PlayerStatus, Status},
    setup::Speed,
    stamina::Stamina,
};

const RUN_SPEED: f32 = 1.5;
const STAMINA_DRAIN: f32 = 0.25; 
const EMPTY: f32 = 0.0;

// --- COMPONENTS --- 
#[derive(Component)]
pub struct DebugText; 

pub fn run(
    player_query: Single<(&mut Stamina, &mut Speed, &mut PlayerStatus), With<Player>>,
    inventory: Single<&Inventory>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut stamina, mut speed, mut player) = player_query.into_inner(); 
    if player.condition != Status::Normal || inventory.searching {
        return; 
    }

    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        speed.current = RUN_SPEED; 
        if stamina.current > EMPTY { stamina.current -= STAMINA_DRAIN; }
        else { 
            stamina.current = EMPTY;
            player.condition = Status::Exhausted;
            speed.current = speed.base; 
        }
    }
    else if keyboard_input.just_released(KeyCode::ShiftLeft) {
        speed.current = speed.base;
    }
}

pub fn prevent_movement (
    inventory: Single<&Inventory>,  
    // mut velocity: Single<&mut Velocity, With<Player>>, 
) {
    if !inventory.searching {
        return; 
    }

    // velocity.linvel = Vec3::ZERO; 
}