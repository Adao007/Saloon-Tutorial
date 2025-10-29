use super::player::Player;
use bevy::prelude::*;

#[derive(Resource)]
pub struct MousePos {
    pub position: Vec2,
}

pub fn get_mouse_position(
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut mouse_pos: ResMut<MousePos>,
) {
    let (camera, camera_transform) = q_camera.single().unwrap();
    let window = q_window.single().unwrap();

    if let Some(world_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        mouse_pos.position = world_pos;
    }
}

pub fn snap_aim(mut player_query: Query<&mut Transform, With<Player>>, mouse_pos: Res<MousePos>) {
    let aim_translation = mouse_pos.position.xy();
    for mut player_transform in &mut player_query {
        let to_aim = (aim_translation - player_transform.translation.xy()).normalize();
        let rotate = Quat::from_rotation_arc(Vec3::Y, to_aim.extend(0.));
        player_transform.rotation = rotate;
    }
}
