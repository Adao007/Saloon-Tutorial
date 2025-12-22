use bevy::prelude::*; 
use crate::gameplay::player::{
    player::{Player, PlayerStatus, Status},
    setup::Speed,
};

const REGEN: f32 = 2.0;
const WALK_SPEED: f32 = 85.0;

#[derive(Component)]
pub struct Stamina{
    pub max: f32,
    pub current: f32, 
}

#[derive(Component)]
pub struct StaminaBar {
    pub entity: Entity,
}

pub fn update_stamina(
    stamina_query: Query<&Stamina>, 
    mut bar_query: Query<(&StaminaBar, &mut Node), With<StaminaBar>>, 
) {
    for (bar, mut node) in &mut bar_query {
        if let Ok(stamina) = stamina_query.get(bar.entity) {
            node.width = Val::Percent(stamina.current / stamina.max * 100.0); 
        }
    }
}

pub fn restore_stamina(
    stamina_query: Single<(&PlayerStatus, &mut Stamina, &Speed), With<Player>>, 
    time: Res<Time>,
) {
    let (status, mut stamina, speed) = stamina_query.into_inner(); 
    if status.condition == Status::Exhausted {
        return; 
    }

    if stamina.current >= 100.0 && speed.current != WALK_SPEED {
        return; 
    }

    stamina.current += REGEN * time.delta_secs(); 
}