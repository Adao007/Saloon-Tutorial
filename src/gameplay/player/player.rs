use bevy::prelude::*;
use crate::gameplay::player::aim::MousePos;
use crate::gameplay::player::setup::spawn_player;
use crate::gameplay::player::aim::get_mouse_position;
use crate::gameplay::player::aim::rotate_aim;
use crate::gameplay::player::movement::run;
use crate::gameplay::player::stamina::update_stamina;
use crate::gameplay::player::health::update_health;
use crate::gameplay::player::stamina::restore_stamina;
use crate::gameplay::player::movement::prevent_movement;
use crate::gameplay::player::stamina::Stamina;

const ZERO: f32 = 0.0;
const SEC: f32 = 1.0; 
const SERVED: f32 = 5.0; 
const RECOVERY: f32 = 20.0; 

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
                run,
                // apply_velocity,
                update_stamina,
                update_health,
                restore_stamina,
                exhaust,
                prevent_movement.after(run),
            ));
    }
}


// --- COMPONENTS ---
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerStatus {
    pub condition: Status,
    pub duration: f32, // Timer for status-effects
}

#[derive(Component, PartialEq)]
pub enum Status {
    Normal,
    Exhausted, 
}

// --- SYSTEMS --- 
fn exhaust (
    mut status: Single<&mut PlayerStatus>,
    mut stamina: Single<&mut Stamina>,
    timer: Res<Time>,  
) {
    if status.condition != Status::Exhausted {
        return; 
    }

    println!("YOU ARE EXHAUSTED!!! TIMEOUT LEFT: {:?} ", (SERVED - status.duration)); 
    if status.duration >= SERVED {
        status.duration = ZERO;
        stamina.current += RECOVERY;
        status.condition = Status::Normal; 
        
    }
    else {
        status.duration += SEC * timer.delta_secs();
    }
}