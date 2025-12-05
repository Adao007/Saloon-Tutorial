use bevy::prelude::*; 
use super::player::Player; 

const REGEN: f32 = 0.15;

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
    stamina_query: Single<&mut Stamina, With<Player>>, 
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut stamina = stamina_query.into_inner(); 

    if stamina.current >= 100.0 || keyboard_input.pressed(KeyCode::ShiftLeft) {
        return; 
    }

    stamina.current += REGEN; 
}