use bevy::prelude::*; 
use super::{camera::*, world::*};

pub struct StagePlugin; 
impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        app 
            .add_systems(Startup, (
                // init_environment, 
                object_setup,
                init_camera,
            ))
            .add_systems(Update, (
                update_camera,
            ));
    }
}