use bevy::prelude::*;

pub fn get_mouse_position(q_window: Query<&Window>, q_camera: Query<(&Camera, &GlobalTransform)>) {
    let (camera, camera_transform) = q_camera.single().unwrap();
    let window = q_window.single().unwrap();

    if let Some(world_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        println!("Screen Position: {}", world_pos);
    }
}
