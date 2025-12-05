use crate::gameplay::player::player::Player; 
use bevy::prelude::*;

const CAMERA_DECAY_RATE: f32 = 2.;

pub fn init_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn update_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}
