use super::setup::*;
use bevy::prelude::*;

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_camera);
    }
}
