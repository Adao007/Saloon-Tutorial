use bevy::prelude::*; 

#[derive(Component)]
pub struct Health {
    pub max: f32,
    pub current: f32, 
}

#[derive(Component)]
pub struct HealthBar {
    pub entity: Entity,
}

pub fn update_health(
    health_query: Query<&Health>, 
    mut bar_query: Query<(&HealthBar, &mut Node), With<HealthBar>>,
) {
    for (bar, mut node) in &mut bar_query {
        if let Ok(health) = health_query.get(bar.entity) {
            node.width = Val::Percent(health.current / health.max * 100.0); 
        }
    }
}