use bevy::prelude::*;

use crate::gameplay::player::setup::InteractionSensor;

pub struct GizmosPlugin; 
impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, draw_interaction_sensors); 
    }
}

fn draw_interaction_sensors(
    mut gizmos: Gizmos, 
    sensor_query: Query<&GlobalTransform, With<InteractionSensor>>,
) {
    for transform in sensor_query.iter() {
        gizmos.circle_2d(
            transform.translation().truncate(), 
            60.0,
            Color::srgba(0.0, 1.0, 0.0, 0.3),
        );
    } 
}